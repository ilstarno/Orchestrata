use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::dto::error_catalog_entry::ErrorCatalogEntry;

use super::error_code::ERROR_CODE_HEADER;

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum LeadPreviewErrorCode {
    InvalidLeadPreviewRequest,
    UnknownLeadExample,
    LeadExamplesUnavailable,
}

impl LeadPreviewErrorCode {
    pub(super) const ALL: [Self; 3] = [
        Self::InvalidLeadPreviewRequest,
        Self::UnknownLeadExample,
        Self::LeadExamplesUnavailable,
    ];

    pub(super) const fn entry(self) -> ErrorCatalogEntry {
        match self {
            Self::InvalidLeadPreviewRequest => ErrorCatalogEntry::new(
                "invalid_lead_preview_request",
                "This endpoint accepts only a built-in example ID and no request body.",
                &[400, 413],
            ),
            Self::UnknownLeadExample => ErrorCatalogEntry::new(
                "unknown_lead_example",
                "No built-in lead example matches the requested ID.",
                &[404],
            ),
            Self::LeadExamplesUnavailable => ErrorCatalogEntry::new(
                "lead_examples_unavailable",
                "The server could not load the built-in lead examples.",
                &[500],
            ),
        }
    }

    pub(super) fn response(self, status: StatusCode) -> Response {
        let entry = self.entry();
        let mut response = (status, entry.description).into_response();
        response
            .headers_mut()
            .insert(ERROR_CODE_HEADER, HeaderValue::from_static(entry.code));
        response
    }
}
