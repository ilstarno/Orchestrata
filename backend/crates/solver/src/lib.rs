#[path = "module.rs"]
mod module;

pub use module::enums::{constraint::Constraint, error::Error};
pub use module::problem::Problem;
pub use module::reference_solver::solve;
pub use module::score::Score;
pub use module::scoring::score;
pub use module::solution::Solution;
pub use module::validation::validate;
pub use module::variable::Variable;
