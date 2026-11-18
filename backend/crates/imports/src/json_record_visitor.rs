use std::fmt;

use serde::de::{Error, MapAccess, Visitor};

use crate::{constants::MAX_COLUMNS, json_record::JsonRecord};

pub(super) struct RecordVisitor;

impl<'de> Visitor<'de> for RecordVisitor {
    type Value = JsonRecord;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON object with scalar fields")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut fields = Vec::new();
        while let Some((name, value)) = map.next_entry()? {
            if fields.len() == MAX_COLUMNS {
                return Err(M::Error::custom("too many fields"));
            }
            fields.push((name, value));
        }
        Ok(JsonRecord(fields))
    }
}
