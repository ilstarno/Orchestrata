use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{Constraint, Variable};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Problem {
    pub variables: Vec<Variable>,
    pub constraints: Vec<Constraint>,
}
