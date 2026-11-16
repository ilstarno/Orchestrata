use axum::{
    Json,
    body::Bytes,
    extract::{Path, rejection::BytesRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use orchestrata_application::lead_preview::{LeadPreview, PreviewError, preview};

use crate::api::lead_preview_error_code::LeadPreviewErrorCode;

#[utoipa::path(
    post,
    path = "/api/v1/examples/lead-to-contact/{lead_id}/preview",
    description = "Evaluate one built-in synthetic lead example without accepting input records, connecting to a CRM, or writing data. Created and reused are preview outcomes only.",
    params(("lead_id" = String, Path, description = "Built-in ID L-001 through L-060")),
    responses(
        (status = 200, description = "Synthetic decision with no side effects", body = LeadPreview),
        (status = 400, description = "The request must not contain a body",
            headers(("X-Orchestrata-Error-Code" = LeadPreviewErrorCode))),
        (status = 404, description = "No matching built-in example",
            headers(("X-Orchestrata-Error-Code" = LeadPreviewErrorCode))),
        (status = 413, description = "The request contains a body beyond the accepted limit",
            headers(("X-Orchestrata-Error-Code" = LeadPreviewErrorCode))),
        (status = 500, description = "The built-in examples are unavailable",
            headers(("X-Orchestrata-Error-Code" = LeadPreviewErrorCode)))
    )
)]
pub async fn evaluate(
    Path(lead_id): Path<String>,
    body: Result<Bytes, BytesRejection>,
) -> Response {
    let status = match body {
        Ok(bytes) if bytes.is_empty() => None,
        Err(error) if error.status() == StatusCode::PAYLOAD_TOO_LARGE => Some(error.status()),
        _ => Some(StatusCode::BAD_REQUEST),
    };
    if let Some(status) = status {
        return LeadPreviewErrorCode::InvalidLeadPreviewRequest.response(status);
    }

    match preview(&lead_id) {
        Ok(result) => Json(result).into_response(),
        Err(PreviewError::UnknownExample) => {
            LeadPreviewErrorCode::UnknownLeadExample.response(StatusCode::NOT_FOUND)
        }
        Err(PreviewError::ExamplesUnavailable) => LeadPreviewErrorCode::LeadExamplesUnavailable
            .response(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
