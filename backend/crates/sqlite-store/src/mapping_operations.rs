use orchestrata_imports::{ImportError, Mapping};
use rusqlite::{OptionalExtension, params};

use crate::{ColumnMappingId, JobOwnerId, SqliteStore, StoreError};

const MAX_MAPPING_BYTES: usize = 65_536;

impl SqliteStore {
    /// Save an immutable mapping snapshot, not CSV rows. The application resolves the owner.
    pub fn save_column_mapping(
        &mut self,
        owner: JobOwnerId,
        mapping: &Mapping,
    ) -> Result<ColumnMappingId, StoreError> {
        validate_mapping(mapping, StoreError::InvalidMapping)?;
        let json = serde_json::to_string(mapping).map_err(|_| StoreError::InvalidMapping)?;
        if json.len() > MAX_MAPPING_BYTES {
            return Err(StoreError::InvalidMapping);
        }

        let id = ColumnMappingId::new();
        self.connection.execute(
            "INSERT INTO column_mappings (mapping_id, owner_id, mapping_json) VALUES (?1, ?2, ?3)",
            params![id.as_uuid().to_string(), owner.as_uuid().to_string(), json],
        )?;
        Ok(id)
    }

    /// A caller cannot distinguish a different user's mapping from a missing mapping.
    pub fn column_mapping(
        &self,
        owner: JobOwnerId,
        id: ColumnMappingId,
    ) -> Result<Option<Mapping>, StoreError> {
        let mut statement = self.connection.prepare(
            "SELECT mapping_json FROM column_mappings WHERE mapping_id = ?1 AND owner_id = ?2",
        )?;
        let stored = statement
            .query_row(
                params![id.as_uuid().to_string(), owner.as_uuid().to_string()],
                |row| {
                    let cell = row.get_ref(0)?;
                    let value = cell
                        .as_str()
                        .ok()
                        .filter(|value| !value.is_empty() && value.len() <= MAX_MAPPING_BYTES)
                        .ok_or(StoreError::InvalidStoredMapping);
                    Ok(value.and_then(decode))
                },
            )
            .optional()?;
        stored.transpose()
    }
}

fn decode(json: &str) -> Result<Mapping, StoreError> {
    let mapping = serde_json::from_str(json).map_err(|_| StoreError::InvalidStoredMapping)?;
    validate_mapping(&mapping, StoreError::InvalidStoredMapping)?;
    Ok(mapping)
}

fn validate_mapping(mapping: &Mapping, invalid_error: StoreError) -> Result<(), StoreError> {
    mapping.validate().map_err(|error| match error {
        ImportError::UnsupportedMappingVersion(version) => {
            StoreError::UnsupportedMappingVersion(version)
        }
        _ => invalid_error,
    })
}
