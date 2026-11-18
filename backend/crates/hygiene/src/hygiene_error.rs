#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HygieneError {
    BlankVariableName(usize),
    DuplicateVariableName(String),
    DuplicateDomainValue(usize),
    DuplicateConstraintVariable,
    EmptyConstraint,
}
