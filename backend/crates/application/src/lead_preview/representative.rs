use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Representative {
    pub id: String,
    pub territory: String,
    pub language: String,
    pub available: bool,
    pub workload: u32,
}
