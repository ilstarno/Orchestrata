mod access_log;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/access_log_tests.rs"]
mod access_log_tests;
#[path = "controllers/module.rs"]
pub mod controllers;
pub mod error;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/error_catalog_tests.rs"]
mod error_catalog_tests;
mod error_code;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/fleet_examples_tests.rs"]
mod fleet_examples_tests;
mod http_error_code;
mod lead_preview_error_code;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/lead_preview_tests.rs"]
mod lead_preview_tests;
pub mod openapi;
mod recipe_catalog;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/recipe_catalog_tests.rs"]
mod recipe_catalog_tests;
mod recipe_grid;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/recipe_grid.rs"]
mod recipe_grid_tests;
#[cfg(test)]
#[path = "../../../../../backend_tests/api/recipe_run_tests.rs"]
mod recipe_run_tests;
mod solve_admission;

mod router_builder;

pub use router_builder::router;
#[cfg(test)]
use router_builder::router_with_admission;
