use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::Response,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::router;

const GRID: &str = "/api/v1/recipes/grid";
const ERROR_CODE: &str = "x-orchestrata-error-code";

async fn request(path: &str) -> Response {
    router()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn json_data(response: Response) -> Value {
    let body = to_bytes(response.into_body(), 128 * 1024).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn ids(value: &Value) -> Vec<&str> {
    value["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn serves_only_builtin_metadata_and_authoritative_column_descriptors() {
    let response = request(GRID).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));
    assert!(!response.headers().contains_key(ERROR_CODE));

    let page = json_data(response).await;
    assert_eq!(page["page"], 1);
    assert_eq!(page["page_size"], 25);
    assert_eq!(
        page["columns"],
        json!([
            {"id": "id", "label": "ID", "sortable": true},
            {"id": "title", "label": "Title", "sortable": true},
            {"id": "scenario", "label": "Scenario", "sortable": false}
        ]),
    );

    let mut catalog = json_data(request("/api/v1/recipes").await)
        .await
        .as_array()
        .unwrap()
        .clone();
    catalog.sort_unstable_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    assert_eq!(page["total_items"], catalog.len());
    let expected: Vec<_> = catalog.iter().map(|recipe| {
        json!({"id": recipe["id"], "title": recipe["title"], "scenario": recipe["scenario"]})
    }).collect();
    assert_eq!(page["items"], json!(expected));

    let details = request("/api/v1/recipes/field-service").await;
    assert_eq!(details.status(), StatusCode::OK);
    assert!(json_data(details).await["inputs"].is_array());
}

#[tokio::test]
async fn filters_by_literal_text_before_paging_and_sorting() {
    let response = request(&format!("{GRID}?q=%20FIELD-SERVICE%20&page_size=1&page=1")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let one = json_data(response).await;
    // The ID matches regardless of the spelling in title or scenario.
    assert_eq!(ids(&one), ["field-service"]);
    assert_eq!(one["total_items"], 1);

    let empty =
        json_data(request(&format!("{GRID}?q=FIELD-SERVICE&page_size=1&page=2")).await).await;
    assert_eq!(empty["page"], 2);
    assert_eq!(empty["total_items"], 1);
    assert_eq!(ids(&empty), Vec::<&str>::new());

    let literal = json_data(request(&format!("{GRID}?q=%25field%25")).await).await;
    assert_eq!(literal["total_items"], 0);
}

#[tokio::test]
async fn returns_stable_nonoverlapping_pages_in_requested_title_order() {
    let all = json_data(request(&format!("{GRID}?sort=title&direction=desc")).await).await;
    let rows = all["items"].as_array().unwrap();
    assert!(rows.len() > 4);
    assert!(rows.windows(2).all(|pair| {
        let title = |i: usize| pair[i]["title"].as_str().unwrap();
        title(0) > title(1)
            || title(0) == title(1) && pair[0]["id"].as_str() < pair[1]["id"].as_str()
    }));

    let first =
        json_data(request(&format!("{GRID}?sort=title&direction=desc&page_size=2")).await).await;
    let second = json_data(
        request(&format!(
            "{GRID}?sort=title&direction=desc&page_size=2&page=2"
        ))
        .await,
    )
    .await;
    assert_eq!(first["total_items"], second["total_items"]);
    let combined: Vec<_> = ids(&first).into_iter().chain(ids(&second)).collect();
    assert_eq!(combined, ids(&all)[..4]);
}

#[tokio::test]
async fn rejects_unknown_duplicate_and_out_of_bounds_controls_without_echoing_inputs() {
    let overlong = format!("q={}", "a".repeat(101));
    let overlong_utf8 = format!("q={}", "%C3%A9".repeat(51));
    for query in [
        "page=0",
        "page=100001",
        "page=-1",
        "page_size=0",
        "page_size=101",
        "page=1&page=2",
        "page=40000000000",
        "sort=scenario",
        "sort=title%20DESC",
        "direction=random",
        "where=unknown",
        "q=%00",
        &overlong,
        &overlong_utf8,
    ] {
        let response = request(&format!("{GRID}?{query}")).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{query}");
        assert_eq!(response.headers()[ERROR_CODE], "request_rejected");
        assert_eq!(
            to_bytes(response.into_body(), 1_024).await.unwrap(),
            "Invalid recipe grid query"
        );
    }
}

#[tokio::test]
async fn documents_grid_controls_and_error_header_in_openapi() {
    let spec = json_data(request("/api/v1/openapi.json").await).await;
    let grid = &spec["paths"][GRID]["get"];
    let names: Vec<_> = grid["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            assert_ne!(entry["required"], true);
            entry["name"].as_str().unwrap()
        })
        .collect();
    assert_eq!(names, ["page", "page_size", "sort", "direction", "q"]);
    assert_eq!(
        grid["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/RecipeGridResponse"
    );
    let parameters = grid["parameters"].as_array().unwrap();
    for (index, default) in [json!(1), json!(25), json!("id"), json!("asc"), json!("")]
        .into_iter()
        .enumerate()
    {
        assert_eq!(parameters[index]["schema"]["default"], default);
    }
    for (index, maximum) in [(0, 100_000), (1, 100)] {
        assert_eq!(parameters[index]["schema"]["minimum"], 1);
        assert_eq!(parameters[index]["schema"]["maximum"], maximum);
    }
    assert_eq!(parameters[4]["schema"]["maxLength"], 100);
    assert!(
        parameters[4]["description"]
            .as_str()
            .unwrap()
            .contains("UTF-8 bytes")
    );

    for (status, code) in [("400", "request_rejected"), ("500", "internal_error")] {
        let response = &grid["responses"][status];
        assert_eq!(
            response["headers"]["X-Orchestrata-Error-Code"]["description"],
            code
        );
        assert_eq!(
            response["content"]["text/plain"]["schema"]["type"],
            "string"
        );
    }
    let schemas = &spec["components"]["schemas"];
    for (collection, element) in [("columns", "RecipeGridColumn"), ("items", "RecipeGridRow")] {
        assert_eq!(
            schemas["RecipeGridResponse"]["properties"][collection]["items"]["$ref"],
            format!("#/components/schemas/{element}")
        );
    }
    assert_eq!(
        schemas["RecipeGridColumnId"]["enum"],
        json!(["id", "title", "scenario"])
    );
    assert_eq!(schemas["RecipeGridSort"]["enum"], json!(["id", "title"]));
}
