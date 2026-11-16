use std::sync::OnceLock;

use orchestrata_recipes::load_recipe;

use crate::dto::recipe_metadata_response::RecipeMetadataResponse;

use super::builtin_recipe::BuiltinRecipe;

macro_rules! example {
    ($id:literal) => {
        BuiltinRecipe {
            id: $id,
            xml: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../recipes/",
                $id,
                ".xml"
            )),
            problem: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../recipes/examples/",
                $id,
                "/problem.json"
            )),
            #[cfg(test)]
            expected: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../recipes/examples/",
                $id,
                "/expected.json"
            )),
        }
    };
}

pub(super) const BUILTIN_RECIPES: &[BuiltinRecipe] = &[
    example!("airline-crew-assignment"),
    example!("airport-gate-assignment"),
    example!("call-center-staffing"),
    example!("cluster-workload-placement"),
    example!("dental-clinic-room-allocation"),
    example!("field-service"),
    example!("fleet-pickup-delivery"),
    example!("healthcare-supply-delivery"),
    example!("high-school-timetabling"),
    example!("hospital-nonclinical-shift-coverage"),
    example!("loading-dock-assignment"),
    example!("operating-room-block-allocation"),
];

static METADATA: OnceLock<Option<Vec<RecipeMetadataResponse>>> = OnceLock::new();

pub(super) fn all() -> Option<&'static [RecipeMetadataResponse]> {
    METADATA
        .get_or_init(|| {
            BUILTIN_RECIPES
                .iter()
                .map(|builtin| {
                    let recipe = load_recipe(builtin.xml).ok()?;
                    if recipe.id() != builtin.id {
                        return None;
                    }
                    Some((&recipe).into())
                })
                .collect()
        })
        .as_deref()
}

pub(super) fn find_example(id: &str) -> Option<&'static str> {
    BUILTIN_RECIPES
        .iter()
        .find(|recipe| recipe.id == id)
        .map(|recipe| recipe.problem)
}
