use std::sync::OnceLock;

use orchestrata_recipes::{builtin_recipe, builtin_recipes, load_recipe};

use crate::dto::recipe_metadata_response::RecipeMetadataResponse;

static METADATA: OnceLock<Option<Vec<RecipeMetadataResponse>>> = OnceLock::new();

pub(super) fn all() -> Option<&'static [RecipeMetadataResponse]> {
    METADATA
        .get_or_init(|| {
            builtin_recipes()
                .iter()
                .map(|builtin| {
                    let recipe = load_recipe(builtin.xml()).ok()?;
                    if recipe.id() != builtin.id() {
                        return None;
                    }
                    Some((&recipe).into())
                })
                .collect()
        })
        .as_deref()
}

pub(super) fn find_example(id: &str) -> Option<&'static str> {
    builtin_recipe(id).map(|recipe| recipe.problem_json())
}
