mod docker_fixture;
mod postgres_fixture;

use std::error::Error;

use orchestrata_session_store::{SessionDigest, SessionStore, SessionStoreError};
use sea_orm::ConnectionTrait;
use uuid::Uuid;

const MIGRATION: &str =
    include_str!("../../backend/migrations/postgres/20261118143736_sessions.sql");

#[tokio::test]
async fn applies_schema_and_persists_digest_lifecycle_in_postgres() -> Result<(), Box<dyn Error>> {
    let fixture = postgres_fixture::PostgresFixture::start().await?;
    let db = fixture.connect().await?;
    db.execute_unprepared(MIGRATION).await?;
    let store = SessionStore::postgres(db)?;
    store.initialize_schema().await?;
    store.initialize_schema().await?;

    let first = SessionDigest::from_bytes([43; 32]);
    let second = SessionDigest::from_bytes([44; 32]);
    let principal = Uuid::from_u128(147);
    store.create(&first, principal, 1_000, 2_000).await?;
    store.create(&second, principal, 1_000, 3_000).await?;
    assert!(matches!(
        store
            .create(&first, Uuid::from_u128(255), 1_000, 2_000)
            .await,
        Err(SessionStoreError::AlreadyExists)
    ));
    assert_eq!(store.resolve(&first, 1_999).await?, Some(principal));
    assert_eq!(store.resolve(&first, 2_000).await?, None);
    assert!(store.revoke(&second).await?);
    assert!(!store.revoke(&second).await?);
    assert_eq!(store.resolve(&second, 2_000).await?, None);
    drop(store);
    let reopened = SessionStore::postgres(fixture.connect().await?)?;
    assert_eq!(reopened.resolve(&first, 1_999).await?, Some(principal));
    assert_eq!(reopened.resolve(&second, 1_999).await?, None);
    drop(reopened);

    let connection = fixture.connect().await?;
    connection
        .execute_unprepared("DROP TABLE app_sessions; CREATE TABLE app_sessions (unrelated TEXT)")
        .await?;
    assert!(connection.execute_unprepared(MIGRATION).await.is_err());
    let incompatible = SessionStore::postgres(fixture.connect().await?)?;
    assert!(matches!(
        incompatible.initialize_schema().await,
        Err(SessionStoreError::StoreUnavailable)
    ));
    connection
        .execute_unprepared("SELECT unrelated FROM app_sessions")
        .await?;
    drop((incompatible, connection));
    fixture.remove()?;
    Ok(())
}
