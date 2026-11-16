use uuid::{Uuid, Variant};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ColumnMappingId(Uuid);

impl ColumnMappingId {
    pub fn parse(value: Uuid) -> Option<Self> {
        (value.get_version_num() == 7 && value.get_variant() == Variant::RFC4122)
            .then_some(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }

    pub(super) fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
