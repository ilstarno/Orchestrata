use orchestrata_recipe_package::PackageError;
use orchestrata_recipe_runtime::mathematical::ModelError;

#[derive(Debug, PartialEq, Eq)]
pub enum RecipeRunError {
    Unauthorized,
    EntitlementRequired,
    UnknownPackage,
    UnknownEntrypoint,
    StaleRevision,
    InvalidPackage,
    InvalidInput,
    CapacityLimit,
    Infeasible,
    ExecutionFailed,
}

impl From<PackageError> for RecipeRunError {
    fn from(value: PackageError) -> Self {
        match value {
            PackageError::Limit | PackageError::Model(ModelError::ResourceLimit) => {
                Self::CapacityLimit
            }
            PackageError::Model(ModelError::InvalidInput) => Self::InvalidInput,
            PackageError::Model(ModelError::Infeasible) => Self::Infeasible,
            PackageError::Model(ModelError::SolverFailure) => Self::ExecutionFailed,
            PackageError::Entrypoint => Self::UnknownEntrypoint,
            _ => Self::InvalidPackage,
        }
    }
}
