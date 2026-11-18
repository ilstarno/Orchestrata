/// Caller-canonicalized text. The digest helper does not validate syntax or normalize these values.
#[derive(Clone, Copy)]
pub enum CanonicalContact<'a> {
    Email(&'a str),
    Phone(&'a str),
}

impl<'a> CanonicalContact<'a> {
    pub(super) fn fields(self) -> (&'static [u8], &'a str) {
        match self {
            Self::Email(value) => (b"email", value),
            Self::Phone(value) => (b"phone", value),
        }
    }
}
