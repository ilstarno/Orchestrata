#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("session store requires PostgreSQL")]
    UnsupportedBackend,
    #[error("expected a nonnegative reference time and a later expiry")]
    InvalidExpiry,
    #[error("expected a nonnegative reference time")]
    InvalidReferenceTime,
    #[error("session digest already exists")]
    AlreadyExists,
    #[error("session store operation failed")]
    StoreUnavailable,
}
