#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("invalid job submission")]
    InvalidSubmission,
    #[error("unsupported store schema version {0}")]
    UnsupportedSchema(i64),
    #[error("invalid persisted job or event data")]
    InvalidRecord,
    #[error("SQLite operation failed")]
    Database(#[from] rusqlite::Error),
}
