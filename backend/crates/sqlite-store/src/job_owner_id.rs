use uuid::{Uuid, Variant};

/// A local user's identity resolved by the application, never a group or request scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct JobOwnerId(Uuid);

impl JobOwnerId {
    pub fn parse(value: Uuid) -> Option<Self> {
        (value.get_version_num() == 7 && value.get_variant() == Variant::RFC4122)
            .then_some(Self(value))
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}
