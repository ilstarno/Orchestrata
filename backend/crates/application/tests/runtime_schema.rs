use orchestrata_application::schema::{
    FieldDefinition, FieldId, FieldKind, ObjectDefinition, ObjectId, RelationCardinality,
    RelationSpec, RuntimeSchema, ScalarKind, SchemaError, TriggerAction, TriggerDefinition,
    TriggerEvent, TriggerId,
};
use uuid::{Uuid, Version};

fn object(key: &str) -> ObjectDefinition {
    ObjectDefinition {
        id: ObjectId::new(),
        key: key.to_owned(),
        version: 1,
        fields: vec![],
        triggers: vec![],
    }
}

fn field(key: &str, kind: FieldKind) -> FieldDefinition {
    FieldDefinition {
        id: FieldId::new(),
        key: key.to_owned(),
        required: true,
        kind,
    }
}

fn trigger(event: TriggerEvent, watched_field: Option<FieldId>) -> TriggerDefinition {
    TriggerDefinition {
        id: TriggerId::new(),
        event,
        watched_field,
        action: TriggerAction::EmitEvent {
            key: "assignment_changed".to_owned(),
        },
    }
}

fn relation(target: ObjectId, cardinality: RelationCardinality) -> FieldKind {
    FieldKind::Relation(RelationSpec {
        target,
        cardinality,
    })
}

#[test]
fn compiles_typed_fields_and_forward_and_self_relations_by_identity() {
    let mut appointment = object("appointment");
    appointment.version = 2;
    let mut technician = object("technician");
    appointment.fields = vec![
        field("location", FieldKind::Scalar(ScalarKind::String)),
        field("duration", FieldKind::Scalar(ScalarKind::Integer)),
        field("confirmed", FieldKind::Scalar(ScalarKind::Boolean)),
        field(
            "technicians",
            relation(technician.id, RelationCardinality::Many),
        ),
    ];
    technician.fields = vec![FieldDefinition {
        required: false,
        ..field(
            "supervisor",
            relation(technician.id, RelationCardinality::One),
        )
    }];
    appointment.triggers = vec![trigger(
        TriggerEvent::Updated,
        Some(appointment.fields[3].id),
    )];

    let schema = RuntimeSchema::compile(vec![appointment.clone(), technician.clone()]).unwrap();

    assert_eq!(schema.definition(appointment.id), Some(&appointment));
    assert_eq!(schema.definition(technician.id), Some(&technician));
    assert_eq!(schema.definition(ObjectId::new()), None);
}

#[test]
fn generated_and_parsed_identifiers_use_uuidv7() {
    let object = ObjectId::new();
    let field = FieldId::new();
    let trigger = TriggerId::new();

    for id in [object.as_uuid(), field.as_uuid(), trigger.as_uuid()] {
        assert_eq!(id.get_version(), Some(Version::SortRand));
    }
    assert_eq!(ObjectId::parse(object.as_uuid()), Some(object));
    assert_eq!(FieldId::parse(field.as_uuid()), Some(field));
    assert_eq!(TriggerId::parse(trigger.as_uuid()), Some(trigger));
    assert_eq!(ObjectId::parse(Uuid::nil()), None);
    assert_eq!(FieldId::parse(Uuid::nil()), None);
    assert_eq!(TriggerId::parse(Uuid::nil()), None);
}

#[test]
fn rejects_missing_relation_and_cross_object_trigger_field() {
    let mut first = object("first");
    let mut second = object("second");
    let target = ObjectId::new();
    let missing = field("target", relation(target, RelationCardinality::One));
    first.fields.push(missing.clone());
    assert_eq!(
        RuntimeSchema::compile(vec![first.clone()]).err(),
        Some(SchemaError::UnknownRelationTarget {
            field: missing.id,
            target
        })
    );

    first.fields.clear();
    let second_field = field("target", FieldKind::Scalar(ScalarKind::Integer));
    second.fields.push(second_field.clone());
    let invalid = trigger(TriggerEvent::Updated, Some(second_field.id));
    first.triggers.push(invalid.clone());
    assert_eq!(
        RuntimeSchema::compile(vec![first, second]).err(),
        Some(SchemaError::UnknownTriggerField {
            trigger: invalid.id,
            field: second_field.id
        })
    );
}

#[test]
fn rejects_duplicate_object_field_and_trigger_identity() {
    let mut first = object("first");
    let mut second = object("second");
    let original = field("floor", FieldKind::Scalar(ScalarKind::Integer));
    let action = trigger(TriggerEvent::Created, None);

    assert_eq!(
        RuntimeSchema::compile(vec![first.clone(), first.clone()]).err(),
        Some(SchemaError::DuplicateObjectId(first.id))
    );

    first.fields.push(original.clone());
    second.fields.push(original.clone());
    assert_eq!(
        RuntimeSchema::compile(vec![first.clone(), second.clone()]).err(),
        Some(SchemaError::DuplicateFieldId(original.id))
    );

    second.fields.clear();
    first.triggers.push(action.clone());
    second.triggers.push(action.clone());
    assert_eq!(
        RuntimeSchema::compile(vec![first, second]).err(),
        Some(SchemaError::DuplicateTriggerId(action.id))
    );
}

#[test]
fn rejects_duplicate_object_and_field_keys_with_distinct_ids() {
    let mut first = object("same");
    let second = object("same");
    assert_eq!(
        RuntimeSchema::compile(vec![first.clone(), second.clone()]).err(),
        Some(SchemaError::DuplicateObjectKey(second.id))
    );

    let original = field("floor", FieldKind::Scalar(ScalarKind::Integer));
    let duplicate = field("floor", FieldKind::Scalar(ScalarKind::String));
    first.fields.extend([original, duplicate.clone()]);
    assert_eq!(
        RuntimeSchema::compile(vec![first]).err(),
        Some(SchemaError::DuplicateFieldKey(duplicate.id))
    );
}

#[test]
fn rejects_non_update_filters_and_duplicate_trigger_declarations() {
    let mut candidate = object("appointment");
    let watched = field("address", FieldKind::Scalar(ScalarKind::String));
    candidate.fields.push(watched.clone());
    let invalid = trigger(TriggerEvent::Deleted, Some(watched.id));
    candidate.triggers.push(invalid.clone());
    assert_eq!(
        RuntimeSchema::compile(vec![candidate.clone()]).err(),
        Some(SchemaError::InvalidTriggerFieldFilter(invalid.id))
    );

    let first = trigger(TriggerEvent::Updated, Some(watched.id));
    let duplicate = trigger(TriggerEvent::Updated, Some(watched.id));
    candidate.triggers = vec![first, duplicate.clone()];
    assert_eq!(
        RuntimeSchema::compile(vec![candidate]).err(),
        Some(SchemaError::DuplicateTrigger(duplicate.id))
    );
}

#[test]
fn rejects_zero_version_and_invalid_unbounded_keys() {
    let mut candidate = object("appointment");
    candidate.version = 0;
    assert_eq!(
        RuntimeSchema::compile(vec![candidate.clone()]).err(),
        Some(SchemaError::InvalidVersion(candidate.id))
    );

    candidate.version = 1;
    candidate.key = "é".to_owned();
    assert_eq!(
        RuntimeSchema::compile(vec![candidate.clone()]).err(),
        Some(SchemaError::InvalidObjectKey(candidate.id))
    );

    candidate.key = "appointment".to_owned();
    let invalid_field = field(&"x".repeat(65), FieldKind::Scalar(ScalarKind::String));
    candidate.fields.push(invalid_field.clone());
    assert_eq!(
        RuntimeSchema::compile(vec![candidate.clone()]).err(),
        Some(SchemaError::InvalidFieldKey(invalid_field.id))
    );

    candidate.fields.clear();
    let invalid_action = TriggerDefinition {
        action: TriggerAction::EmitEvent {
            key: "https://example.invalid/task".to_owned(),
        },
        ..trigger(TriggerEvent::Created, None)
    };
    candidate.triggers.push(invalid_action.clone());
    assert_eq!(
        RuntimeSchema::compile(vec![candidate]).err(),
        Some(SchemaError::InvalidTriggerEventKey(invalid_action.id))
    );
}

#[test]
fn enforces_catalog_and_per_definition_bounds() {
    assert_eq!(
        RuntimeSchema::compile(
            (0..129)
                .map(|index| object(&format!("obj_{index}")))
                .collect()
        )
        .err(),
        Some(SchemaError::ObjectLimitExceeded)
    );
    let mut candidate = object("appointment");
    candidate.fields = (0..65)
        .map(|index| {
            field(
                &format!("field_{index}"),
                FieldKind::Scalar(ScalarKind::Integer),
            )
        })
        .collect();
    assert_eq!(
        RuntimeSchema::compile(vec![candidate.clone()]).err(),
        Some(SchemaError::FieldLimitExceeded(candidate.id))
    );
    candidate.fields.clear();
    candidate.triggers = (0..33)
        .map(|_| trigger(TriggerEvent::Created, None))
        .collect();
    assert_eq!(
        RuntimeSchema::compile(vec![candidate.clone()]).err(),
        Some(SchemaError::TriggerLimitExceeded(candidate.id))
    );
}
