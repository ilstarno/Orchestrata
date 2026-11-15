use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::Score;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Solution {
    pub values: Vec<i64>,
    pub score: Score,
    pub evaluated: u64,
}
