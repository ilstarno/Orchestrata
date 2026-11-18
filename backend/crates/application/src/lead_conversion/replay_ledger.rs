use std::sync::Mutex;

use crate::lead_preview;

use super::{ConversionError, ConversionEvent, WriteIntent, ledger_state::LedgerState};

/// An in-memory synthetic fixture, not persistence or a live connector.
#[derive(Default)]
pub struct ReplayLedger {
    state: Mutex<LedgerState>,
}

impl ReplayLedger {
    /// Commits one conversion per example within the life of this fixture.
    pub fn apply(
        &self,
        lead_id: &str,
        intent: WriteIntent,
    ) -> Result<ConversionEvent, ConversionError> {
        if intent != WriteIntent::ApplyToFixture {
            return Err(ConversionError::ExplicitIntentRequired);
        }
        let decision = lead_preview::preview(lead_id).map_err(ConversionError::Preview)?;
        let proposed = ConversionEvent::from_preview(decision)?;
        Ok(self.lock()?.record(proposed))
    }

    pub fn events(&self) -> Result<Vec<ConversionEvent>, ConversionError> {
        Ok(self.lock()?.events.values().cloned().collect())
    }

    pub fn created_contact_count(&self) -> Result<usize, ConversionError> {
        Ok(self.lock()?.created_contacts.len())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, LedgerState>, ConversionError> {
        self.state
            .lock()
            .map_err(|_| ConversionError::LedgerUnavailable)
    }
}
