use super::{FieldId, ObjectId, TriggerId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaError {
    ObjectLimitExceeded,
    DuplicateObjectId(ObjectId),
    InvalidObjectKey(ObjectId),
    DuplicateObjectKey(ObjectId),
    InvalidVersion(ObjectId),
    FieldLimitExceeded(ObjectId),
    DuplicateFieldId(FieldId),
    InvalidFieldKey(FieldId),
    DuplicateFieldKey(FieldId),
    UnknownRelationTarget { field: FieldId, target: ObjectId },
    TriggerLimitExceeded(ObjectId),
    DuplicateTriggerId(TriggerId),
    InvalidTriggerFieldFilter(TriggerId),
    UnknownTriggerField { trigger: TriggerId, field: FieldId },
    InvalidTriggerEventKey(TriggerId),
    DuplicateTrigger(TriggerId),
}
