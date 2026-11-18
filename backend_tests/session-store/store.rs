use std::collections::BTreeMap;

use orchestrata_session_store::{SessionDigest, SessionStore, SessionStoreError};
use sea_orm::{DbBackend, MockDatabase, MockExecResult, Value};
use uuid::Uuid;

#[tokio::test]
async fn writes_digest_resolves_an_active_principal_and_revokes_idempotently() {
    let principal = Uuid::from_u128(123);
    let digest_bytes = [19; 32];
    let session = BTreeMap::from([
        ("session_digest", Value::from(digest_bytes.to_vec())),
        ("principal_id", Value::from(principal)),
        ("expires_at_unix_ms", Value::from(2_000_i64)),
    ]);
    let result = |rows_affected| MockExecResult {
        rows_affected,
        last_insert_id: 0,
    };
    let database = MockDatabase::new(DbBackend::Postgres)
        .append_exec_results([result(1), result(1), result(0)])
        .append_query_results([vec![session], vec![]])
        .into_connection();
    let store = SessionStore::postgres(database.clone()).unwrap();
    let digest = SessionDigest::from_bytes(digest_bytes);
    store
        .create(&digest, principal, 1_000, 2_000)
        .await
        .unwrap();
    assert_eq!(
        store.resolve(&digest, 1_001).await.unwrap(),
        Some(principal)
    );
    assert_eq!(store.resolve(&digest, 2_000).await.unwrap(), None);
    assert!(store.revoke(&digest).await.unwrap());
    assert!(!store.revoke(&digest).await.unwrap());

    let operations = database.into_transaction_log();
    let created = &operations[0].statements()[0];
    assert!(created.sql.starts_with("INSERT INTO \"app_sessions\""));
    assert!(created.sql.contains("\"session_digest\""));
    assert!(
        created
            .values
            .as_ref()
            .unwrap()
            .0
            .contains(&Value::from(digest_bytes.to_vec()))
    );
    for operation in &operations[1..3] {
        let lookup = &operation.statements()[0];
        assert!(lookup.sql.contains("\"session_digest\" = $1"));
        assert!(lookup.sql.contains("\"expires_at_unix_ms\" > $2"));
    }
}

#[tokio::test]
async fn rejects_invalid_times_without_touching_the_database() {
    assert!(matches!(
        SessionStore::postgres(MockDatabase::new(DbBackend::Sqlite).into_connection()),
        Err(SessionStoreError::UnsupportedBackend)
    ));
    let database = MockDatabase::new(DbBackend::Postgres).into_connection();
    let store = SessionStore::postgres(database.clone()).unwrap();
    let digest = SessionDigest::from_bytes([31; 32]);
    assert!(matches!(
        store.create(&digest, Uuid::nil(), 200, 200).await,
        Err(SessionStoreError::InvalidExpiry)
    ));
    assert!(matches!(
        store.resolve(&digest, -1).await,
        Err(SessionStoreError::InvalidReferenceTime)
    ));
    assert!(database.into_transaction_log().is_empty());
}

#[tokio::test]
async fn unavailable_database_returns_only_the_redacted_store_error() {
    let database = MockDatabase::new(DbBackend::Postgres)
        .append_query_errors([sea_orm::DbErr::Custom("synthetic database detail".into())])
        .into_connection();
    let store = SessionStore::postgres(database).unwrap();
    let error = store
        .resolve(&SessionDigest::from_bytes([17; 32]), 1_000)
        .await
        .unwrap_err();
    assert!(matches!(error, SessionStoreError::StoreUnavailable));
    assert_eq!(error.to_string(), "session store operation failed");
}
