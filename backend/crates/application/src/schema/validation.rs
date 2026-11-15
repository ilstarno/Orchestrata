use std::collections::HashSet;

use super::{
    FieldId, FieldKind, ObjectDefinition, ObjectId, SchemaError, TriggerAction, TriggerEvent,
    TriggerId,
    constants::{MAX_FIELDS, MAX_KEY_BYTES, MAX_OBJECTS, MAX_TRIGGERS},
};

pub(super) fn validate(definitions: &[ObjectDefinition]) -> Result<(), SchemaError> {
    if definitions.len() > MAX_OBJECTS {
        return Err(SchemaError::ObjectLimitExceeded);
    }
    let mut object_ids = HashSet::new();
    let mut object_keys = HashSet::new();
    for object in definitions {
        if !object_ids.insert(object.id) {
            return Err(SchemaError::DuplicateObjectId(object.id));
        }
        if !valid_key(&object.key) {
            return Err(SchemaError::InvalidObjectKey(object.id));
        }
        if !object_keys.insert(&object.key) {
            return Err(SchemaError::DuplicateObjectKey(object.id));
        }
        if object.version == 0 {
            return Err(SchemaError::InvalidVersion(object.id));
        }
    }

    let mut field_ids = HashSet::new();
    let mut trigger_ids = HashSet::new();
    for object in definitions {
        let object_fields = validate_fields(object, &object_ids, &mut field_ids)?;
        validate_triggers(object, &object_fields, &mut trigger_ids)?;
    }
    Ok(())
}

fn validate_fields(
    object: &ObjectDefinition,
    object_ids: &HashSet<ObjectId>,
    global_ids: &mut HashSet<FieldId>,
) -> Result<HashSet<FieldId>, SchemaError> {
    if object.fields.len() > MAX_FIELDS {
        return Err(SchemaError::FieldLimitExceeded(object.id));
    }
    let mut keys = HashSet::new();
    let mut ids = HashSet::new();
    for field in &object.fields {
        if !global_ids.insert(field.id) {
            return Err(SchemaError::DuplicateFieldId(field.id));
        }
        ids.insert(field.id);
        if !valid_key(&field.key) {
            return Err(SchemaError::InvalidFieldKey(field.id));
        }
        if !keys.insert(&field.key) {
            return Err(SchemaError::DuplicateFieldKey(field.id));
        }
        if let FieldKind::Relation(relation) = field.kind
            && !object_ids.contains(&relation.target)
        {
            return Err(SchemaError::UnknownRelationTarget {
                field: field.id,
                target: relation.target,
            });
        }
    }
    Ok(ids)
}

fn validate_triggers(
    object: &ObjectDefinition,
    object_fields: &HashSet<FieldId>,
    global_ids: &mut HashSet<TriggerId>,
) -> Result<(), SchemaError> {
    if object.triggers.len() > MAX_TRIGGERS {
        return Err(SchemaError::TriggerLimitExceeded(object.id));
    }
    let mut declarations = HashSet::new();
    for trigger in &object.triggers {
        if !global_ids.insert(trigger.id) {
            return Err(SchemaError::DuplicateTriggerId(trigger.id));
        }
        if let Some(field) = trigger.watched_field {
            if trigger.event != TriggerEvent::Updated {
                return Err(SchemaError::InvalidTriggerFieldFilter(trigger.id));
            }
            if !object_fields.contains(&field) {
                return Err(SchemaError::UnknownTriggerField {
                    trigger: trigger.id,
                    field,
                });
            }
        }
        let TriggerAction::EmitEvent { key } = &trigger.action;
        if !valid_key(key) {
            return Err(SchemaError::InvalidTriggerEventKey(trigger.id));
        }
        if !declarations.insert((trigger.event, trigger.watched_field, &trigger.action)) {
            return Err(SchemaError::DuplicateTrigger(trigger.id));
        }
    }
    Ok(())
}

fn valid_key(key: &str) -> bool {
    if key.len() > MAX_KEY_BYTES {
        return false;
    }
    let mut bytes = key.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
