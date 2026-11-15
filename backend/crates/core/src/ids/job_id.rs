use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "Uuid", into = "Uuid")]
pub struct JobId(Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn parse(value: Uuid) -> Result<Self, CoreError> {
        if value.get_version_num() != 7 {
            return Err(CoreError::InvalidIdentifier);
        }
        Ok(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<Uuid> for JobId {
    type Error = CoreError;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<JobId> for Uuid {
    fn from(value: JobId) -> Self {
        value.0
    }
}
