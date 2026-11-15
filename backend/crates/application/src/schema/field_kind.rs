use super::{RelationSpec, ScalarKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Scalar(ScalarKind),
    Relation(RelationSpec),
}
