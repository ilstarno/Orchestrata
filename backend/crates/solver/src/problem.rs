use serde::{Deserialize, Serialize};

use crate::{Constraint, Variable};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Problem {
    pub variables: Vec<Variable>,
    pub constraints: Vec<Constraint>,
}
