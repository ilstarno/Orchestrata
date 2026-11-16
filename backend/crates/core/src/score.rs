use serde::{Deserialize, Serialize};

use crate::errors::CoreError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HardMediumSoftScore {
    pub hard: i64,
    pub medium: i64,
    pub soft: i64,
}

impl HardMediumSoftScore {
    pub fn is_feasible(self) -> bool {
        self.hard >= 0
    }

    pub fn checked_add(self, other: Self) -> Result<Self, CoreError> {
        Ok(Self {
            hard: self
                .hard
                .checked_add(other.hard)
                .ok_or(CoreError::ScoreOverflow)?,
            medium: self
                .medium
                .checked_add(other.medium)
                .ok_or(CoreError::ScoreOverflow)?,
            soft: self
                .soft
                .checked_add(other.soft)
                .ok_or(CoreError::ScoreOverflow)?,
        })
    }
}
