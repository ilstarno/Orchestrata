#[derive(schemars::JsonSchema, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeId {
    /// A recipe identifier returned by list_recipes.
    pub id: String,
}
