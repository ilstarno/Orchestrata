use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JsonScalar {
    Null,
    Boolean(bool),
    Integer(i64),
    Text(String),
}
