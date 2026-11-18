use super::RunScope;

/// Only a trusted, composed backend session verifier can implement this
/// boundary. A request's user, role or workspace assertions are not a session.
pub trait SessionAuthority {
    type Session: ?Sized;

    fn may_run(&self, session: &Self::Session, scope: RunScope<'_>) -> bool;
}
