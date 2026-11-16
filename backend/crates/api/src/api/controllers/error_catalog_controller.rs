use axum::Json;

use crate::{
    api::{error_code::SolveErrorCode, http_error_code::HttpErrorCode},
    dto::error_catalog_entry::ErrorCatalogEntry,
};

#[utoipa::path(
    get,
    path = "/api/v1/error-codes",
    responses((status = 200, description = "Stable HTTP error codes and their descriptions", body = [ErrorCatalogEntry]))
)]
pub async fn error_catalog() -> Json<Vec<ErrorCatalogEntry>> {
    Json(
        SolveErrorCode::ALL
            .into_iter()
            .map(SolveErrorCode::entry)
            .chain(HttpErrorCode::ALL.into_iter().map(HttpErrorCode::entry))
            .collect(),
    )
}
