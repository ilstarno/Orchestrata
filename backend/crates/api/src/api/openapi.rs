use orchestrata_application::lead_preview::{LeadExampleCatalog, LeadPreview, Outcome, Reason};
use orchestrata_solver::{Constraint, Problem, Score, Solution, Variable};
use utoipa::OpenApi;

use crate::{
    api::{
        controllers::{
            error_catalog_controller, health_controller, lead_preview_controller,
            recipe_controller, recipe_grid_controller, solve_controller,
        },
        error_code::SolveErrorCode,
        lead_preview_error_code::LeadPreviewErrorCode,
    },
    dto::{
        error_catalog_entry::ErrorCatalogEntry, health_response::HealthResponse,
        recipe_field_kind::RecipeFieldKind, recipe_field_response::RecipeFieldResponse,
        recipe_grid_column::RecipeGridColumn, recipe_grid_column_id::RecipeGridColumnId,
        recipe_grid_direction::RecipeGridDirection, recipe_grid_response::RecipeGridResponse,
        recipe_grid_row::RecipeGridRow, recipe_grid_sort::RecipeGridSort,
        recipe_metadata_response::RecipeMetadataResponse, solve_response::SolveResponse,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        error_catalog_controller::error_catalog,
        health_controller::health,
        lead_preview_controller::catalog,
        lead_preview_controller::evaluate,
        recipe_controller::list,
        recipe_grid_controller::grid,
        recipe_controller::metadata,
        recipe_controller::example,
        recipe_controller::run_example,
        solve_controller::solve_problem
    ),
    components(schemas(
        HealthResponse,
        SolveResponse,
        SolveErrorCode,
        ErrorCatalogEntry,
        LeadExampleCatalog,
        LeadPreview,
        Outcome,
        Reason,
        LeadPreviewErrorCode,
        RecipeGridResponse,
        RecipeGridColumn,
        RecipeGridColumnId,
        RecipeGridRow,
        RecipeGridSort,
        RecipeGridDirection,
        RecipeMetadataResponse,
        RecipeFieldResponse,
        RecipeFieldKind,
        Problem,
        Variable,
        Constraint,
        Solution,
        Score
    ))
)]
pub struct ApiDoc;
