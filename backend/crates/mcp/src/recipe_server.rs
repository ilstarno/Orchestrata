use orchestrata_application::lead_preview::{PreviewError, example_catalog, preview};
use orchestrata_recipes::{
    BuiltinRecipe, FieldKind, FieldSpec, builtin_recipe, builtin_recipes, load_recipe,
};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool, tool_router,
};
use serde_json::{Value, json};

use crate::{
    empty_lead_catalog_request::EmptyLeadCatalogRequest, lead_example_id::LeadExampleId,
    recipe_id::RecipeId,
};

#[derive(Clone)]
pub struct RecipeServer;

#[tool_router(server_handler)]
impl RecipeServer {
    #[tool(
        description = "List identifiers for built-in synthetic lead-to-contact previews.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn list_lead_examples(
        &self,
        Parameters(_request): Parameters<EmptyLeadCatalogRequest>,
    ) -> CallToolResult {
        lead_result(example_catalog())
    }

    #[tool(
        description = "Evaluate one built-in synthetic lead example by ID. Does not accept leads, connect to CRM, or write records.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn preview_lead_example(
        &self,
        Parameters(request): Parameters<LeadExampleId>,
    ) -> CallToolResult {
        lead_result(preview(&request.lead_id))
    }

    #[tool(
        description = "List the built-in synthetic planning recipes.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn list_recipes(&self) -> CallToolResult {
        let recipes = builtin_recipes()
            .iter()
            .map(|entry| {
                let recipe = load_recipe(entry.xml()).ok()?;
                Some(json!({ "id": recipe.id(), "title": recipe.title() }))
            })
            .collect::<Option<Vec<_>>>();
        match recipes {
            Some(recipes) => CallToolResult::structured(json!({ "recipes": recipes })),
            None => catalog_unavailable(),
        }
    }

    #[tool(
        description = "Read the scenario and declared input and output fields of a built-in recipe.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn describe_recipe(&self, Parameters(id): Parameters<RecipeId>) -> CallToolResult {
        let Some(entry) = find(&id.id) else {
            return unknown_recipe();
        };
        let Ok(recipe) = load_recipe(entry.xml()) else {
            return catalog_unavailable();
        };
        CallToolResult::structured(json!({
            "id": recipe.id(),
            "title": recipe.title(),
            "scenario": recipe.scenario(),
            "inputs": recipe.inputs().iter().map(field).collect::<Vec<_>>(),
            "outputs": recipe.outputs().iter().map(field).collect::<Vec<_>>(),
        }))
    }

    #[tool(
        description = "Get the synthetic normalized problem for a built-in recipe. Does not run a solver.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn get_recipe_example(&self, Parameters(id): Parameters<RecipeId>) -> CallToolResult {
        let Some(entry) = find(&id.id) else {
            return unknown_recipe();
        };
        let Ok(problem) = serde_json::from_str::<Value>(entry.problem_json()) else {
            return catalog_unavailable();
        };
        CallToolResult::structured(json!({
            "id": entry.id(),
            "synthetic": true,
            "problem": problem,
        }))
    }
}

fn find(id: &str) -> Option<&'static BuiltinRecipe> {
    if id.len() > 64
        || !id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return None;
    }
    builtin_recipe(id)
}

fn field(field: &FieldSpec) -> Value {
    let kind = match field.kind {
        FieldKind::String => "string",
        FieldKind::Integer => "integer",
        FieldKind::Boolean => "boolean",
        FieldKind::StringList => "string-list",
        FieldKind::IntegerList => "integer-list",
    };
    json!({
        "name": field.name,
        "type": kind,
        "required": field.required,
        "description": field.description,
    })
}

fn unknown_recipe() -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(
        "Recipe not found. Use list_recipes to find a built-in identifier.",
    )])
}

fn catalog_unavailable() -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(
        "The built-in recipe catalog is unavailable.",
    )])
}

fn lead_result(result: Result<impl serde::Serialize, PreviewError>) -> CallToolResult {
    match result {
        Ok(value) => match serde_json::to_value(value) {
            Ok(content) => CallToolResult::structured(content),
            Err(_) => lead_error(PreviewError::ExamplesUnavailable),
        },
        Err(error) => lead_error(error),
    }
}

fn lead_error(error: PreviewError) -> CallToolResult {
    let message = match error {
        PreviewError::UnknownExample => "No built-in lead example matches the requested ID.",
        PreviewError::ExamplesUnavailable => "The built-in lead examples are unavailable.",
    };
    let mut result = CallToolResult::error(vec![ContentBlock::text(message)]);
    result.structured_content = Some(json!({ "code": error.code() }));
    result
}
