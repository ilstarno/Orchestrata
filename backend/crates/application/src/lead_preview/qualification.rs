use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Qualification {
    pub need: bool,
    pub person: bool,
    pub next_step: bool,
}
