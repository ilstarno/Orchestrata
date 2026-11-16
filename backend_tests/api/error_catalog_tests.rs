use std::collections::HashSet;

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use super::router;

async fn request(method: Method, path: &str, body: &str) -> axum::response::Response {
    router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn documented_codes_are_unique_and_include_current_transport_errors() {
    let response = request(Method::GET, "/api/v1/error-codes", "").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(!response.headers().contains_key("x-orchestrata-error-code"));
    let body = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
    let catalog: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(catalog.len(), 9);
    let mut codes = HashSet::new();
    for entry in &catalog {
        assert!(codes.insert(entry["code"].as_str().unwrap()));
        assert!(!entry["description"].as_str().unwrap().is_empty());
    }
    for (method, path, body, status, code) in [
        (
            Method::GET,
            "/unknown",
            "",
            StatusCode::NOT_FOUND,
            "not_found",
        ),
        (
            Method::POST,
            "/api/v1/health",
            "",
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
        ),
        (
            Method::POST,
            "/api/v1/solve",
            "{",
            StatusCode::BAD_REQUEST,
            "invalid_request_body",
        ),
        (
            Method::POST,
            "/api/v1/solve",
            r#"{"variables":[{"name":"", "domain":[1]}],"constraints":[]}"#,
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_problem",
        ),
    ] {
        let response = request(method, path, body).await;
        assert_eq!(response.status(), status);
        assert_eq!(response.headers()["x-orchestrata-error-code"], code);
        assert!(codes.contains(code));
        assert!(response.headers().contains_key("x-request-id"));
    }
}
