#[derive(Clone, Copy)]
pub enum ContactLookupPurpose {
    DirectoryLookup,
    DuplicateDetection,
}

impl ContactLookupPurpose {
    pub(super) fn tag(self) -> &'static [u8] {
        match self {
            Self::DirectoryLookup => b"directory-lookup",
            Self::DuplicateDetection => b"duplicate-detection",
        }
    }
}
