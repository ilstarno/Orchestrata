use serde::Serialize;

use super::Objective;

/// Review values for an authorized, explicitly selected entrypoint.
#[derive(Debug, Serialize)]
pub struct PackageSelection {
    pub package_id: String,
    pub entrypoint: String,
    pub content_revision: String,
    pub execution_revision: String,
    pub locale: String,
    pub title: String,
    pub description: String,
    pub objective: Objective,
}
