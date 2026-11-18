use std::collections::BTreeMap;

use super::Objective;

/// Trusted installation configuration, never constructed from a run request.
/// Entries bind user-visible meanings to exact model revisions.
pub struct PackageRegistration<'a> {
    pub workspace: &'a str,
    pub package_id: &'a str,
    pub content_revision: &'a str,
    pub requires_entitlement: bool,
    pub entrypoints: BTreeMap<String, (String, Objective)>,
}
