use serde::{Deserialize, Deserializer};

use crate::{JsonScalar, json_record_visitor::RecordVisitor};

pub(super) struct JsonRecord(pub Vec<(String, JsonScalar)>);

impl<'de> Deserialize<'de> for JsonRecord {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(RecordVisitor)
    }
}
