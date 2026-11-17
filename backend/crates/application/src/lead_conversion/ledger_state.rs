use std::collections::{BTreeMap, BTreeSet};

use crate::lead_preview::Outcome;

use super::ConversionEvent;

#[derive(Default)]
pub(super) struct LedgerState {
    pub(super) events: BTreeMap<String, ConversionEvent>,
    pub(super) created_contacts: BTreeSet<String>,
}

impl LedgerState {
    pub(super) fn record(&mut self, proposed: ConversionEvent) -> ConversionEvent {
        if let Some(recorded) = self.events.get(proposed.lead_id()) {
            return recorded.clone();
        }
        if proposed.outcome() == Outcome::Created {
            self.created_contacts
                .insert(proposed.contact_id().to_owned());
        }
        self.events
            .insert(proposed.lead_id().to_owned(), proposed.clone());
        proposed
    }
}
