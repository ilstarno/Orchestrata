use serde::Deserialize;

use super::{contact::Contact, lead::Lead, representative::Representative};

#[derive(Deserialize)]
pub(super) struct Example {
    pub lead: Lead,
    pub representatives: Vec<Representative>,
    pub contacts: Vec<Contact>,
}
