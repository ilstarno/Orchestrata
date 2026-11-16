use std::{error::Error, time::Duration};

use orchestrata_session_store::{SessionDigest, SessionStore, SessionStoreError};
use sea_orm::{ConnectOptions, Database};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, core::IntoContainerPort, runners::AsyncRunner},
};
use uuid::Uuid;

#[tokio::test]
async fn applies_schema_and_persists_digest_lifecycle_in_postgres() -> Result<(), Box<dyn Error>> {
    let password = Uuid::new_v4().simple().to_string();
    let container = Postgres::default()
        .with_db_name("orchestrata_test")
        .with_user("orchestrata")
        .with_password(&password)
        .with_tag("17-alpine")
        .with_mapped_port(0, 5432.tcp())
        .with_host_config_modifier(|config| {
            if let Some(ports) = config.port_bindings.as_mut() {
                for bindings in ports.values_mut().flatten() {
                    for binding in bindings {
                        binding.host_ip = Some("127.0.0.1".into());
                        binding.host_port = Some(String::new());
                    }
                }
            }
        })
        .start()
        .await?;
    let port = container.get_host_port_ipv4(5432.tcp()).await?;
    let mut options = ConnectOptions::new(format!(
        "postgres://orchestrata:{password}@127.0.0.1:{port}/orchestrata_test?sslmode=disable"
    ));
    options
        .connect_timeout(Duration::from_secs(5))
        .sqlx_logging(false);
    let db = Database::connect(options)
        .await
        .map_err(|_| "test database connection failed")?;
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
    Ok(())
}
