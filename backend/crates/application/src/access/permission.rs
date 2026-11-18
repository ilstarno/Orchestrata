use super::AccessAction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Permission {
    Solver(AccessAction),
    Agent(AccessAction),
}
