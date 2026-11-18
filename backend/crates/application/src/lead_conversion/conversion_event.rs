use crate::lead_preview::{LeadPreview, Outcome};

use super::ConversionError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversionEvent {
    lead_id: String,
    contact_id: String,
    outcome: Outcome,
}

impl ConversionEvent {
    pub(super) fn from_preview(decision: LeadPreview) -> Result<Self, ConversionError> {
        if !matches!(decision.outcome, Outcome::Created | Outcome::Reused) {
            return Err(ConversionError::NotConvertible(decision.outcome));
        }
        let contact_id = decision
            .contact_id
            .ok_or(ConversionError::InvalidDecision)?;
        Ok(Self {
            lead_id: decision.lead_id,
            contact_id,
            outcome: decision.outcome,
        })
    }

    pub fn lead_id(&self) -> &str {
        &self.lead_id
    }
    pub fn contact_id(&self) -> &str {
        &self.contact_id
    }
    pub fn outcome(&self) -> Outcome {
        self.outcome
    }
}
