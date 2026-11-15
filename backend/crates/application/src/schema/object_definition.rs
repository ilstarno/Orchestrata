use super::{FieldDefinition, ObjectId, TriggerDefinition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDefinition {
    pub id: ObjectId,
    pub key: String,
    pub version: u32,
    pub fields: Vec<FieldDefinition>,
    pub triggers: Vec<TriggerDefinition>,
}
