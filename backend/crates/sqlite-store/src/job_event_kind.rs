#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobEventKind {
    Submitted,
    Cancelled,
    Completed,
}

impl JobEventKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Submitted => "job_submitted",
            Self::Cancelled => "job_cancelled",
            Self::Completed => "job_completed",
        }
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            "job_submitted" => Some(Self::Submitted),
            "job_cancelled" => Some(Self::Cancelled),
            "job_completed" => Some(Self::Completed),
            _ => None,
        }
    }
}
