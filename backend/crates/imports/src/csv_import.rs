use std::collections::{BTreeMap, HashMap};

use csv::{Reader, ReaderBuilder, StringRecord};

use crate::{
    ColumnMapping, ImportError, Mapping,
    constants::{MAX_BYTES, MAX_CELL_BYTES, MAX_COLUMNS, MAX_ROWS},
    header::{normalized_header, validate_target},
};

pub fn suggest_csv_mapping(data: &[u8], target_fields: &[&str]) -> Result<Mapping, ImportError> {
    if !(1..=MAX_COLUMNS).contains(&target_fields.len()) {
        return Err(ImportError::InvalidMappingCount);
    }
    let mut reader = reader(data)?;
    let headers = reader.headers().map_err(csv_error)?;
    let by_name = index_headers(headers)?;

    let mut columns = Vec::with_capacity(target_fields.len());
    for &target in target_fields {
        validate_target(target)?;
        let index = *by_name
            .get(target)
            .ok_or_else(|| ImportError::MissingHeader(target.to_owned()))?;
        columns.push(ColumnMapping::new(&headers[index], target));
    }
    Mapping::new(columns)
}

pub fn normalize_csv(
    data: &[u8],
    mapping: &Mapping,
) -> Result<Vec<BTreeMap<String, String>>, ImportError> {
    mapping.validate()?;
    let mut reader = reader(data)?;
    let headers = reader.headers().map_err(csv_error)?;
    let by_name = index_headers(headers)?;
    let columns = mapped_indices(mapping, &by_name)?;

    let mut output = Vec::new();
    for record in reader.records() {
        let record = record.map_err(csv_error)?;
        if output.len() == MAX_ROWS {
            return Err(ImportError::TooManyRows);
        }
        for (column, cell) in record.iter().enumerate() {
            if cell.len() > MAX_CELL_BYTES {
                return Err(ImportError::CellTooLarge {
                    line: record.position().map(csv::Position::line),
                    column: column + 1,
                });
            }
        }
        let row = columns
            .iter()
            .map(|(index, target)| (target.to_string(), record[*index].trim().to_owned()))
            .collect();
        output.push(row);
    }
    Ok(output)
}

fn reader(data: &[u8]) -> Result<Reader<&[u8]>, ImportError> {
    if data.len() > MAX_BYTES {
        return Err(ImportError::SourceTooLarge);
    }
    Ok(ReaderBuilder::new()
        .has_headers(true)
        .flexible(false)
        .from_reader(data))
}

fn index_headers(headers: &StringRecord) -> Result<HashMap<String, usize>, ImportError> {
    if headers.is_empty() {
        return Err(ImportError::NoHeaders);
    }
    if headers.len() > MAX_COLUMNS {
        return Err(ImportError::TooManyColumns);
    }
    let mut indexed = HashMap::with_capacity(headers.len());
    for (index, header) in headers.iter().enumerate() {
        let normalized = normalized_header(header)?;
        if indexed.contains_key(&normalized) {
            return Err(ImportError::DuplicateHeader(normalized));
        }
        indexed.insert(normalized, index);
    }
    Ok(indexed)
}

fn mapped_indices<'a>(
    mapping: &'a Mapping,
    headers: &HashMap<String, usize>,
) -> Result<Vec<(usize, &'a str)>, ImportError> {
    mapping
        .columns()
        .iter()
        .map(|column| {
            let source = normalized_header(column.source())?;
            let index = headers
                .get(&source)
                .ok_or(ImportError::MissingHeader(source))?;
            Ok((*index, column.target()))
        })
        .collect()
}

fn csv_error(error: csv::Error) -> ImportError {
    ImportError::InvalidCsv {
        line: error.position().map(csv::Position::line),
    }
}
