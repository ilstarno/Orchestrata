//! Internal recipe-package execution. No transport or authority is installed here.
mod error;
mod input_identity;
mod objective;
mod objective_result;
mod package_registration;
mod package_run;
mod package_selection;
mod package_service;
mod registered_package;
mod run_request;
mod run_scope;
mod session_authority;

pub use error::RecipeRunError;
pub use input_identity::InputIdentity;
pub use objective::Objective;
pub use objective_result::ObjectiveResult;
pub use package_registration::PackageRegistration;
pub use package_run::PackageRun;
pub use package_selection::PackageSelection;
pub use package_service::PackageService;
pub use run_request::RunRequest;
pub use run_scope::RunScope;
pub use session_authority::SessionAuthority;
