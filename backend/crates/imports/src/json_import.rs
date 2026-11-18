use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use serde::{
    Deserializer,
    de::{DeserializeSeed, Error, SeqAccess, Visitor},
};

use crate::{
    ImportError, JsonScalar, Mapping,
    constants::{MAX_BYTES, MAX_CELL_BYTES, MAX_ROWS},
    header::normalized_header,
    json_record::JsonRecord,
};

/// Normalizes a byte-limited array of flat records from an already obtained response.
/// Text, booleans, null and signed 64-bit integers retain their original types.
/// Decimals, arrays and objects are outside this record format.
pub fn normalize_json_records(
    data: &[u8],
    mapping: &Mapping,
) -> Result<Vec<BTreeMap<String, JsonScalar>>, ImportError> {
    mapping.validate()?;
    if data.len() > MAX_BYTES {
        return Err(ImportError::SourceTooLarge);
    }

    let mut deserializer = serde_json::Deserializer::from_slice(data);
    let records = RecordsVisitor
        .deserialize(&mut deserializer)
        .map_err(json_error)?;
    deserializer.end().map_err(json_error)?;
    records
        .into_iter()
        .map(|record| normalize_record(record, mapping))
        .collect()
}

fn normalize_record(
    record: JsonRecord,
    mapping: &Mapping,
) -> Result<BTreeMap<String, JsonScalar>, ImportError> {
    let mut indexed = HashMap::with_capacity(record.0.len());
    for (column, (name, value)) in record.0.into_iter().enumerate() {
        let name = normalized_header(&name)?;
        if indexed.contains_key(&name) {
            return Err(ImportError::DuplicateHeader(name));
        }
        if let JsonScalar::Text(text) = &value
            && text.len() > MAX_CELL_BYTES
        {
            return Err(ImportError::CellTooLarge {
                line: None,
                column: column + 1,
            });
        }
        indexed.insert(name, value);
    }

    mapping
        .columns()
        .iter()
        .map(|column| {
            let source = normalized_header(column.source())?;
            let value = indexed
                .remove(&source)
                .ok_or(ImportError::MissingHeader(source))?;
            Ok((column.target().to_owned(), value))
        })
        .collect()
}

fn json_error(error: serde_json::Error) -> ImportError {
    ImportError::InvalidJson { line: error.line() }
}

struct RecordsVisitor;

impl<'de> DeserializeSeed<'de> for RecordsVisitor {
    type Value = Vec<JsonRecord>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for RecordsVisitor {
    type Value = Vec<JsonRecord>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an array of JSON objects")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut records = Vec::new();
        while let Some(record) = seq.next_element()? {
            if records.len() == MAX_ROWS {
                return Err(S::Error::custom("too many records"));
            }
            records.push(record);
        }
        Ok(records)
    }
}
