use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus { Pending, Running, Completed, Failed, Cancelled }

impl JobStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!((self, next),
            (Self::Pending, Self::Running | Self::Cancelled)
            | (Self::Running, Self::Completed | Self::Failed | Self::Cancelled))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_states_do_not_restart() {
        for state in [JobStatus::Completed, JobStatus::Failed, JobStatus::Cancelled] {
            assert!(!state.can_transition_to(JobStatus::Running));
        }
    }
}
