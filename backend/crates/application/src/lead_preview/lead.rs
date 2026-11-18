use serde::Deserialize;

use super::qualification::Qualification;

#[derive(Deserialize)]
pub(super) struct Lead {
    pub id: String,
    pub company: String,
    pub account: String,
    pub territory: String,
    pub language: String,
    pub email: String,
    pub qualification: Qualification,
}
