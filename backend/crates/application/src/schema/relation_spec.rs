use super::{ObjectId, RelationCardinality};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationSpec {
    pub target: ObjectId,
    pub cardinality: RelationCardinality,
}
