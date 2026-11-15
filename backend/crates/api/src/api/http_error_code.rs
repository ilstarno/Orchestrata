use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::dto::error_catalog_entry::ErrorCatalogEntry;

use super::error_code::ERROR_CODE_HEADER;

#[derive(Clone, Copy)]
pub(super) enum HttpErrorCode {
    NotFound,
    MethodNotAllowed,
    RequestRejected,
    InternalError,
}

impl HttpErrorCode {
    pub(super) const ALL: [Self; 4] = [
        Self::NotFound,
        Self::MethodNotAllowed,
        Self::RequestRejected,
        Self::InternalError,
    ];

    fn for_status(status: StatusCode) -> Option<Self> {
        match status {
            StatusCode::NOT_FOUND => Some(Self::NotFound),
            StatusCode::METHOD_NOT_ALLOWED => Some(Self::MethodNotAllowed),
            _ if status.is_client_error() => Some(Self::RequestRejected),
            _ if status.is_server_error() => Some(Self::InternalError),
            _ => None,
        }
    }

    pub(super) const fn entry(self) -> ErrorCatalogEntry {
        match self {
            Self::NotFound => ErrorCatalogEntry::new(
                "not_found",
                "No resource matches the requested route.",
                &[404],
            ),
            Self::MethodNotAllowed => ErrorCatalogEntry::new(
                "method_not_allowed",
                "The route does not accept this HTTP method.",
                &[405],
            ),
            Self::RequestRejected => ErrorCatalogEntry::new(
                "request_rejected",
                "The server rejected the request without a more specific code. See the HTTP status.",
                &[],
            ),
            Self::InternalError => ErrorCatalogEntry::new(
                "internal_error",
                "The server could not complete the request. Use the request ID when reporting the failure.",
                &[],
            ),
        }
    }
}

pub(super) async fn attach_unclassified_code(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    if !response.headers().contains_key(&ERROR_CODE_HEADER)
        && let Some(error) = HttpErrorCode::for_status(response.status())
    {
        response.headers_mut().insert(
            ERROR_CODE_HEADER,
            HeaderValue::from_static(error.entry().code),
        );
    }
    response
}
