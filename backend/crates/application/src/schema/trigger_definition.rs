use super::{FieldId, TriggerAction, TriggerEvent, TriggerId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerDefinition {
    pub id: TriggerId,
    pub event: TriggerEvent,
    pub watched_field: Option<FieldId>,
    pub action: TriggerAction,
}
