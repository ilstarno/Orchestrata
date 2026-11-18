use orchestrata_recipe_package::PackageError;
use orchestrata_recipe_runtime::mathematical::ModelError;

use super::RecipeRunError;

#[derive(Debug, PartialEq, Eq)]
pub enum PenaltyRunError {
    Package(RecipeRunError),
    UnsupportedModel,
    UnsupportedPackage,
    UnitMismatch,
    InvalidResult,
}

impl From<RecipeRunError> for PenaltyRunError {
    fn from(error: RecipeRunError) -> Self {
        Self::Package(error)
    }
}

impl From<PackageError> for PenaltyRunError {
    fn from(error: PackageError) -> Self {
        match error {
            PackageError::Unsupported => Self::UnsupportedPackage,
            PackageError::Model(ModelError::UnsupportedSemantics) => Self::UnsupportedModel,
            _ => Self::Package(error.into()),
        }
    }
}
