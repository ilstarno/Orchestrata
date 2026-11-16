#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewError {
    UnknownExample,
    ExamplesUnavailable,
}

impl PreviewError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnknownExample => "unknown_lead_example",
            Self::ExamplesUnavailable => "lead_examples_unavailable",
        }
    }
}
