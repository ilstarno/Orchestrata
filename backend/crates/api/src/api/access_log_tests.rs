use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode, header::CONTENT_TYPE},
};
use serde_json::Value;
use tower::ServiceExt;
use uuid::{Uuid, Version};

use crate::bootloader::Bootloader;

#[derive(Default)]
struct AccessLogBuffer(Mutex<Vec<u8>>);

impl Write for &AccessLogBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn access_events_correlate_requests_without_recording_client_data() {
    let logs = Arc::new(AccessLogBuffer::default());
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(Arc::clone(&logs))
        .finish();
    let _subscriber = tracing::subscriber::set_default(subscriber);
    let app = Bootloader::build();
    let cases = [
        (
            Method::GET,
            "/api/v1/health?access_token=PRIVATE_QUERY",
            "",
            StatusCode::OK,
            "/api/v1/health",
            "INFO",
        ),
        (
            Method::GET,
            "/PRIVATE_UNMATCHED_PATH",
            "",
            StatusCode::NOT_FOUND,
            "<unmatched>",
            "INFO",
        ),
        (
            Method::POST,
            "/api/v1/solve",
            "PRIVATE_MALFORMED_BODY",
            StatusCode::BAD_REQUEST,
            "/api/v1/solve",
            "INFO",
        ),
        (
            Method::POST,
            "/api/v1/solve",
            r#"{"variables":[{"name":"PRIVATE_PROBLEM_NAME","domain":[0]}],"constraints":[{"kind":"prefer_value","variable":0,"value":1,"soft_weight":9223372036854775807},{"kind":"prefer_value","variable":0,"value":2,"soft_weight":2}]}"#,
            StatusCode::INTERNAL_SERVER_ERROR,
            "/api/v1/solve",
            "ERROR",
        ),
    ];
    let mut request_ids = Vec::new();
    for (method, path, body, status, route, level) in cases {
        let request = Request::builder()
            .uri(path)
            .method(method.clone())
            .header(CONTENT_TYPE, "application/json")
            .header("x-request-id", "PRIVATE_CLIENT_ID")
            .header("authorization", "Bearer PRIVATE_AUTH_TOKEN")
            .body(Body::from(body))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), status);
        let id = response.headers()["x-request-id"].to_str().unwrap();
        assert_eq!(
            Uuid::parse_str(id).unwrap().get_version(),
            Some(Version::SortRand)
        );
        assert!(!request_ids.iter().any(|previous| previous == id));
        request_ids.push(id.to_owned());

        let output = logs.0.lock().unwrap();
        let text = std::str::from_utf8(&output).unwrap();
        assert!(!text.contains("PRIVATE_"), "request data leaked into log");
        assert_eq!(text.lines().count(), request_ids.len());
        let log: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
        assert_eq!(log["level"], level);
        let fields = &log["fields"];
        assert_eq!(fields["event"], "http_access");
        assert_eq!(fields["request_id"], id);
        assert_eq!(fields["status"], status.as_u16());
        assert_eq!(fields["method"], method.as_str());
        assert_eq!(fields["route"], route);
        assert!(fields["duration_ms"].as_f64().unwrap() >= 0.0);
    }
}
