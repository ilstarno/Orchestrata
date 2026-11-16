use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    http::{HeaderName, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::Instrument;
use uuid::Uuid;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
const UNMATCHED_ROUTE: &str = "<unmatched>";

pub(super) async fn observe_request(request: Request, next: Next) -> Response {
    let request_id = Uuid::now_v7().hyphenated().to_string();
    let method = known_method(request.method());
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or(UNMATCHED_ROUTE, MatchedPath::as_str)
        .to_owned();
    let started = Instant::now();

    // Used for correlation, not authorization or idempotency. No request header is trusted.
    let span = tracing::info_span!("http.request", %request_id);
    let mut response = next.run(request).instrument(span).await;
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).expect("a UUID contains only valid header ASCII"),
    );

    emit_access_event(&request_id, method, &route, response.status(), started);
    response
}

fn emit_access_event(
    request_id: &str,
    method: &str,
    route: &str,
    status: StatusCode,
    started: Instant,
) {
    let status = status.as_u16();
    let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
    match status {
        500..=502 | 504..=599 => {
            tracing::error!(
                event = "http_access",
                request_id,
                method,
                route,
                status,
                duration_ms
            );
        }
        503 => {
            tracing::warn!(
                event = "http_access",
                request_id,
                method,
                route,
                status,
                duration_ms
            );
        }
        _ => {
            tracing::info!(
                event = "http_access",
                request_id,
                method,
                route,
                status,
                duration_ms
            );
        }
    }
}

fn known_method(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::HEAD => "HEAD",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::DELETE => "DELETE",
        Method::CONNECT => "CONNECT",
        Method::OPTIONS => "OPTIONS",
        Method::TRACE => "TRACE",
        Method::PATCH => "PATCH",
        _ => "OTHER",
    }
}

#[cfg(test)]
#[path = "../../../../../backend_tests/api/access_log_unit.rs"]
mod tests;
