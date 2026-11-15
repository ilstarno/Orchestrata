use orchestrata_domain::JobStatus;

#[test]
fn terminal_states_do_not_restart() {
    for state in [
        JobStatus::Completed,
        JobStatus::Failed,
        JobStatus::Cancelled,
    ] {
        assert!(!state.can_transition_to(JobStatus::Running));
    }
}
