use super::{FieldId, FieldKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDefinition {
    pub id: FieldId,
    pub key: String,
    pub kind: FieldKind,
    pub required: bool,
}
