use crate::lead_preview::{Outcome, PreviewError};

#[derive(Debug, PartialEq, Eq)]
pub enum ConversionError {
    ExplicitIntentRequired,
    Preview(PreviewError),
    NotConvertible(Outcome),
    InvalidDecision,
    LedgerUnavailable,
}
