use super::UserId;

/// Ownership within one installation. Shared resources require an explicit grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceOwner {
    Personal(UserId),
    Shared,
}
