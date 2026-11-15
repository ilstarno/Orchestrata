use uuid::{Uuid, Variant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId(Uuid);

impl FieldId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn parse(value: Uuid) -> Option<Self> {
        (value.get_version_num() == 7 && value.get_variant() == Variant::RFC4122)
            .then_some(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for FieldId {
    fn default() -> Self {
        Self::new()
    }
}
