#[derive(schemars::JsonSchema, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeadExampleId {
    /// A built-in identifier returned by list_lead_examples.
    pub lead_id: String,
}
