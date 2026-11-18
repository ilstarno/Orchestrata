use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    ColumnMapping, ImportError,
    constants::{MAPPING_VERSION, MAX_COLUMNS},
    header::{normalized_header, validate_target},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    version: u16,
    columns: Vec<ColumnMapping>,
}

impl Mapping {
    pub fn new(columns: Vec<ColumnMapping>) -> Result<Self, ImportError> {
        let mapping = Self {
            version: MAPPING_VERSION,
            columns,
        };
        mapping.validate()?;
        Ok(mapping)
    }

    pub fn columns(&self) -> &[ColumnMapping] {
        &self.columns
    }

    pub fn validate(&self) -> Result<(), ImportError> {
        if self.version != MAPPING_VERSION {
            return Err(ImportError::UnsupportedMappingVersion(self.version));
        }
        if !(1..=MAX_COLUMNS).contains(&self.columns.len()) {
            return Err(ImportError::InvalidMappingCount);
        }

        let mut sources = HashSet::with_capacity(self.columns.len());
        let mut targets = HashSet::with_capacity(self.columns.len());
        for column in &self.columns {
            let source = normalized_header(column.source())?;
            if sources.contains(&source) {
                return Err(ImportError::DuplicateSource(source));
            }
            sources.insert(source);

            let target = column.target();
            validate_target(target)?;
            if !targets.insert(target) {
                return Err(ImportError::DuplicateTarget(target.to_owned()));
            }
        }
        Ok(())
    }
}
