use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub hard: i64,
    pub soft: i64,
}

impl Score {
    pub(crate) fn better_than(self, other: Self) -> bool {
        (self.hard, self.soft) > (other.hard, other.soft)
    }
}
