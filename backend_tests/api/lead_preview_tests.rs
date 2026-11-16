use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    response::Response,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::router;

const PREFIX: &str = "/api/v1/examples/lead-to-contact";
const ERROR_CODE: &str = "x-orchestrata-error-code";
const EXPECTED: &str = include_str!("lead-preview-expected.json");

async fn request(method: Method, path: &str, body: Body) -> Response {
    router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn json_body(response: Response) -> Value {
    let body = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn all_built_in_cases_match_the_independent_walkthrough() {
    let expected: Vec<Value> = serde_json::from_str(EXPECTED).unwrap();
    assert_eq!(expected.len(), 60);
    for want in expected {
        let lead_id = want["lead_id"].as_str().unwrap();
        let response = request(
            Method::POST,
            &format!("{PREFIX}/{lead_id}/preview"),
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK, "{lead_id}");
        assert_eq!(response.headers()["content-type"], "application/json");
        assert!(response.headers().contains_key("x-request-id"));
        assert!(!response.headers().contains_key(ERROR_CODE));
        let actual = json_body(response).await;
        assert_eq!(actual, want, "{lead_id}");
        assert!(!actual.to_string().contains('@'));
    }
}

#[tokio::test]
async fn repeat_requests_return_the_same_preview_without_creating_state() {
    let path = format!("{PREFIX}/L-001/preview");
    let first = json_body(request(Method::POST, &path, Body::empty()).await).await;
    let second = json_body(request(Method::POST, &path, Body::empty()).await).await;
    assert_eq!(first, second);
    assert_eq!(
        first,
        json!({"lead_id":"L-001","outcome":"created","reason":"new",
        "assigned_representative_id":"REP-08","contact_id":"CT-001"})
    );
}

#[tokio::test]
async fn refuses_request_bodies_and_unknown_examples_with_indexed_errors() {
    let valid = format!("{PREFIX}/L-001/preview");
    for (body, expected) in [
        (Body::from("{}"), StatusCode::BAD_REQUEST),
        (Body::from("x".repeat(64)), StatusCode::PAYLOAD_TOO_LARGE),
    ] {
        let response = request(Method::POST, &valid, body).await;
        assert_eq!(response.status(), expected);
        assert_eq!(
            response.headers()[ERROR_CODE],
            "invalid_lead_preview_request"
        );
    }
    for invalid in ["L-000", "L-061", "l-001", "arbitrary"] {
        let response = request(
            Method::POST,
            &format!("{PREFIX}/{invalid}/preview"),
            Body::empty(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()[ERROR_CODE], "unknown_lead_example");
    }
}

#[tokio::test]
async fn openapi_and_error_catalog_publish_the_bodyless_contract() {
    let spec = json_body(request(Method::GET, "/api/v1/openapi.json", Body::empty()).await).await;
    let post = &spec["paths"][format!("{PREFIX}/{{lead_id}}/preview")]["post"];
    assert!(post.get("requestBody").is_none());
    assert_eq!(
        post["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/LeadPreview"
    );
    for code in ["400", "404", "413", "500"] {
        assert!(post["responses"][code]["headers"]["X-Orchestrata-Error-Code"].is_object());
    }
    let schema = &spec["components"]["schemas"]["LeadPreview"];
    assert!(
        schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("contact_id"))
    );
    assert!(
        schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("assigned_representative_id"))
    );
    let catalog = json_body(request(Method::GET, "/api/v1/error-codes", Body::empty()).await).await;
    for (code, statuses) in [
        ("invalid_lead_preview_request", json!([400, 413])),
        ("unknown_lead_example", json!([404])),
        ("lead_examples_unavailable", json!([500])),
    ] {
        let entry = catalog
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["code"] == code)
            .unwrap();
        assert_eq!(entry["status_codes"], statuses);
        assert!(!entry["description"].as_str().unwrap().is_empty());
    }
}

#[tokio::test]
async fn rest_discovery_and_web_views_use_the_same_sample_catalog_and_decision() {
    let expected: Vec<Value> = serde_json::from_str(EXPECTED).unwrap();
    let ids = expected
        .iter()
        .map(|example| example["lead_id"].clone())
        .collect::<Vec<_>>();
    let response = request(Method::GET, PREFIX, Body::empty()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await,
        json!({"synthetic": true, "lead_ids": ids})
    );

    let html_prefix = "/examples/lead-to-contact";
    let response = request(Method::GET, html_prefix, Body::empty()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["content-type"],
        "text/html; charset=utf-8"
    );
    let markup = to_bytes(response.into_body(), 32 * 1024).await.unwrap();
    let markup = str::from_utf8(&markup).unwrap();
    assert!(markup.contains("Synthetic preview only"));
    assert_eq!(markup.matches("<li>").count(), expected.len());
    for id in &ids {
        let id = id.as_str().unwrap();
        assert!(markup.contains(&format!(r#"href="{html_prefix}/{id}""#)));
    }

    for (id, outcome, representative, contact) in [
        ("L-001", "Created", "REP-08", "CT-001"),
        ("L-002", "Reused", "REP-18", "CT-EX-002"),
        ("L-003", "Held", "REP-08", "None"),
        ("L-004", "Review", "None", "None"),
    ] {
        let response = request(Method::GET, &format!("{html_prefix}/{id}"), Body::empty()).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 32 * 1024).await.unwrap();
        let body = str::from_utf8(&body).unwrap();
        assert!(body.contains(&format!("<dt>Outcome</dt><dd>{outcome}</dd>")));
        assert!(body.contains(&format!(
            "<dt>Assigned representative</dt><dd>{representative}</dd>"
        )));
        assert!(body.contains(&format!(
            "<dt>Simulated contact identifier</dt><dd>{contact}</dd>"
        )));
        assert!(!body.contains('@'));
    }

    let unknown = request(
        Method::GET,
        &format!("{html_prefix}/%3Cscript%3E"),
        Body::empty(),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    assert_eq!(unknown.headers()[ERROR_CODE], "unknown_lead_example");
    let body = to_bytes(unknown.into_body(), 1024).await.unwrap();
    assert!(!str::from_utf8(&body).unwrap().contains("<script>"));

    let spec = json_body(request(Method::GET, "/api/v1/openapi.json", Body::empty()).await).await;
    assert_eq!(
        spec["paths"][PREFIX]["get"]["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/LeadExampleCatalog",
    );
    assert!(
        spec["paths"][PREFIX]["get"]["responses"]["500"]["headers"]["X-Orchestrata-Error-Code"]
            .is_object()
    );
}
