use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, QueryFilter, TransactionTrait,
};
use uuid::Uuid;

use crate::{SessionDigest, SessionStoreError, record};

const SCHEMA: &str = include_str!("../../../schema/postgres/app_sessions.sql");

/// Persistence only. It does not authenticate, issue tokens, or manage browser cookies.
pub struct SessionStore {
    connection: DatabaseConnection,
}

impl SessionStore {
    pub fn postgres(connection: DatabaseConnection) -> Result<Self, SessionStoreError> {
        if connection.get_database_backend() != DatabaseBackend::Postgres {
            return Err(SessionStoreError::UnsupportedBackend);
        }
        Ok(Self { connection })
    }

    /// Installs the table and its expiry index. Call using a migration connection, not per request.
    pub async fn initialize_schema(&self) -> Result<(), SessionStoreError> {
        let transaction = self.connection.begin().await.map_err(unavailable)?;
        transaction
            .execute_unprepared(SCHEMA)
            .await
            .map_err(unavailable)?;
        transaction.commit().await.map_err(unavailable)
    }

    /// Persists the digest for a newly issued session. Times are trusted server inputs.
    pub async fn create(
        &self,
        digest: &SessionDigest,
        principal_id: Uuid,
        now_unix_ms: i64,
        expires_at_unix_ms: i64,
    ) -> Result<(), SessionStoreError> {
        if now_unix_ms < 0 || expires_at_unix_ms <= now_unix_ms {
            return Err(SessionStoreError::InvalidExpiry);
        }
        record::Entity::insert(record::ActiveModel {
            session_digest: Set(digest.bytes().to_vec()),
            principal_id: Set(principal_id),
            expires_at_unix_ms: Set(expires_at_unix_ms),
        })
        .exec_without_returning(&self.connection)
        .await
        .map_err(|error| match error.sql_err() {
            Some(sea_orm::SqlErr::UniqueConstraintViolation(_)) => SessionStoreError::AlreadyExists,
            _ => SessionStoreError::StoreUnavailable,
        })?;
        Ok(())
    }

    pub async fn resolve(
        &self,
        digest: &SessionDigest,
        now_unix_ms: i64,
    ) -> Result<Option<Uuid>, SessionStoreError> {
        if now_unix_ms < 0 {
            return Err(SessionStoreError::InvalidReferenceTime);
        }
        record::Entity::find_by_id(digest.bytes().to_vec())
            .filter(record::Column::ExpiresAtUnixMs.gt(now_unix_ms))
            .one(&self.connection)
            .await
            .map(|value| value.map(|session| session.principal_id))
            .map_err(unavailable)
    }

    /// Deletes the persisted session. A repeated revocation is a no-op.
    pub async fn revoke(&self, digest: &SessionDigest) -> Result<bool, SessionStoreError> {
        record::Entity::delete_by_id(digest.bytes().to_vec())
            .exec(&self.connection)
            .await
            .map(|result| result.rows_affected == 1)
            .map_err(unavailable)
    }
}

fn unavailable(_: sea_orm::DbErr) -> SessionStoreError {
    SessionStoreError::StoreUnavailable
}
