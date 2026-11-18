use sea_orm::entity::prelude::*;

pub use crate::relation::Relation;

#[derive(Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "app_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_digest: Vec<u8>,
    pub principal_id: Uuid,
    pub expires_at_unix_ms: i64,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionModel")
            .finish_non_exhaustive()
    }
}

impl ActiveModelBehavior for ActiveModel {}
