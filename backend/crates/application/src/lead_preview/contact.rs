use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Contact {
    pub id: String,
    pub account: String,
    pub email: String,
    pub owner: String,
}
