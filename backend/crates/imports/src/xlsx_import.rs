use std::{
    collections::{BTreeMap, HashMap},
    io::Cursor,
};

use calamine::{Reader, Xlsx};

use crate::{
    ImportError, Mapping,
    constants::{MAX_COLUMNS, MAX_ROWS},
    header::normalized_header,
    xlsx_archive::validate_archive,
    xlsx_value::value_text,
};

const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

/// Imports an explicitly named XLSX worksheet using row 1 as text headers. Empty worksheet
/// rows are skipped; missing mapped cells become empty strings, like empty CSV cells.
/// Accepts a bounded, values-only workbook: no formulas, external relationships or binary parts.
pub fn normalize_xlsx(
    data: &[u8],
    worksheet: &str,
    mapping: &Mapping,
) -> Result<Vec<BTreeMap<String, String>>, ImportError> {
    mapping.validate()?;
    if worksheet.trim().is_empty()
        || worksheet.chars().count() > 31
        || worksheet.chars().any(char::is_control)
    {
        return Err(ImportError::InvalidWorksheet);
    }
    validate_archive(data)?;

    let mut workbook: Xlsx<_> =
        Xlsx::new(Cursor::new(data)).map_err(|_| ImportError::InvalidWorkbook)?;
    if !workbook
        .sheet_names()
        .iter()
        .any(|sheet| sheet == worksheet)
    {
        return Err(ImportError::WorksheetNotFound);
    }
    let mut reader = workbook
        .worksheet_cells_reader(worksheet)
        .map_err(|_| ImportError::InvalidWorkbook)?;
    let mut output = Vec::new();
    let mut output_bytes = 0;
    let mut selected = None;
    let mut last = None;
    let mut active_row = 0;
    let mut values: [String; MAX_COLUMNS] = std::array::from_fn(|_| String::new());
    while let Some(cell) = reader
        .next_cell()
        .map_err(|_| ImportError::InvalidWorkbook)?
    {
        let (row, col) = cell.get_position();
        if row as usize > MAX_ROWS || col as usize >= MAX_COLUMNS {
            return Err(ImportError::WorkbookDimensions);
        }
        if last.is_some_and(|previous| previous >= (row, col)) {
            return Err(ImportError::InvalidWorkbook);
        }
        last = Some((row, col));

        if row != active_row {
            if active_row == 0 {
                selected = Some(mapped_columns(mapping, &values)?);
            } else if let Some(columns) = selected.as_ref() {
                push_row(&mut output, &mut output_bytes, columns, &values)?;
            }
            values = std::array::from_fn(|_| String::new());
            active_row = row;
        }
        values[col as usize] = value_text(cell.get_value(), row == 0, row, col)?;
    }
    if let Some(columns) = selected {
        push_row(&mut output, &mut output_bytes, &columns, &values)?;
    } else {
        mapped_columns(mapping, &values)?;
    }
    Ok(output)
}

fn mapped_columns<'a>(
    mapping: &'a Mapping,
    values: &[String; MAX_COLUMNS],
) -> Result<Vec<(usize, &'a str)>, ImportError> {
    let mut headers = HashMap::with_capacity(MAX_COLUMNS);
    for (index, header) in values.iter().enumerate() {
        if header.is_empty() {
            continue;
        }
        if headers.insert(normalized_header(header)?, index).is_some() {
            return Err(ImportError::DuplicateWorkbookHeader);
        }
    }
    if headers.is_empty() {
        return Err(ImportError::NoHeaders);
    }
    mapping
        .columns()
        .iter()
        .map(|column| {
            let index = headers
                .get(&normalized_header(column.source())?)
                .ok_or(ImportError::MissingWorkbookHeader)?;
            Ok((*index, column.target()))
        })
        .collect()
}

fn push_row(
    output: &mut Vec<BTreeMap<String, String>>,
    budget: &mut usize,
    columns: &[(usize, &str)],
    values: &[String; MAX_COLUMNS],
) -> Result<(), ImportError> {
    if values.iter().all(String::is_empty) {
        return Ok(());
    }
    let bytes: usize = columns
        .iter()
        .map(|(index, target)| target.len() + values[*index].len())
        .sum();
    *budget += bytes;
    if *budget > MAX_OUTPUT_BYTES {
        return Err(ImportError::WorkbookOutputTooLarge);
    }
    output.push(
        columns
            .iter()
            .map(|(index, target)| (target.to_string(), values[*index].clone()))
            .collect(),
    );
    Ok(())
}
