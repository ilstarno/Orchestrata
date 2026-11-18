use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    response::Response,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{router, router_with_admission, solve_admission::SolveAdmission};

const RUN: &str = "/api/v1/recipes/field-service/run-example";
const ERROR_CODE: &str = "x-orchestrata-error-code";

async fn request(app: Router, method: Method, path: &str, body: &'static str) -> Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(path)
            .body(Body::from(body))
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
async fn solves_only_the_checked_in_example() {
    let actual = request(router(), Method::POST, RUN, "").await;
    assert_eq!(actual.status(), StatusCode::OK);
    assert!(!actual.headers().contains_key(ERROR_CODE));
    assert!(actual.headers().contains_key("x-request-id"));
    let actual = json_body(actual).await;
    let expected: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../recipes/examples/field-service/expected.json"
    )))
    .unwrap();
    assert_eq!(actual, json!({"solution": expected}));

    let arbitrary_body = r#"{"variables":[{"name":"injected","domain":[999]}],"constraints":[]}"#;
    let response = request(router(), Method::POST, RUN, arbitrary_body).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json_body(response).await, actual);
}

#[tokio::test]
async fn indexes_unknown_recipe_and_unsupported_method() {
    let response = request(
        router(),
        Method::POST,
        "/api/v1/recipes/missing/run-example",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()[ERROR_CODE], "not_found");

    let response = request(router(), Method::GET, RUN, "").await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.headers()[ERROR_CODE], "method_not_allowed");
}

#[tokio::test]
async fn recipe_runs_use_the_same_admission_as_generic_solves() {
    let app = router_with_admission(SolveAdmission::new(0));
    for path in [RUN, "/api/v1/solve"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"variables":[],"constraints":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()[ERROR_CODE], "solver_busy");
    }
}

#[tokio::test]
async fn openapi_exposes_a_bodyless_run_with_indexed_errors() {
    let response = request(router(), Method::GET, "/api/v1/openapi.json", "").await;
    assert_eq!(response.status(), StatusCode::OK);
    let spec = json_body(response).await;
    let operation = &spec["paths"]["/api/v1/recipes/{recipe_id}/run-example"]["post"];
    assert!(operation.get("requestBody").is_none());
    assert_eq!(
        operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SolveResponse",
    );
    for status in ["404", "500", "503"] {
        assert!(operation["responses"][status]["headers"]["X-Orchestrata-Error-Code"].is_object());
    }
}
