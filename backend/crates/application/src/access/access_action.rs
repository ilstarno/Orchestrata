#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessAction {
    View,
    Run,
    Manage,
}
