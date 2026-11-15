use super::{OrganizationId, UserId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceOwner {
    Personal(UserId),
    Organization(OrganizationId),
}
