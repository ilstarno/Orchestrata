#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("invalid job submission")]
    InvalidSubmission,
    #[error("lease duration must be 1 millisecond to 24 hours")]
    InvalidLeaseDuration,
    #[error("invalid result reference")]
    InvalidResultReference,
    #[error("unsupported store schema version {0}")]
    UnsupportedSchema(i64),
    #[error("database is not an initialized Orchestrata job store or an empty SQLite database")]
    IncompatibleDatabase,
    #[error("invalid persisted job or event data")]
    InvalidRecord,
    #[error("invalid column mapping")]
    InvalidMapping,
    #[error("invalid persisted column mapping")]
    InvalidStoredMapping,
    #[error("unsupported column mapping version {0}")]
    UnsupportedMappingVersion(u16),
    #[error("SQLite operation failed")]
    Database(#[from] rusqlite::Error),
}
