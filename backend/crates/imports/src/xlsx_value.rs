use calamine::DataRef;

use crate::{ImportError, constants::MAX_CELL_BYTES};

pub(crate) fn value_text(
    value: &DataRef<'_>,
    header: bool,
    row: u32,
    col: u32,
) -> Result<String, ImportError> {
    let invalid = || ImportError::UnsupportedWorkbookCell {
        row: row + 1,
        column: col + 1,
    };
    let text = match value {
        DataRef::Empty => String::new(),
        DataRef::SharedString(value) => checked_text(value, row, col)?,
        DataRef::String(value) => checked_text(value, row, col)?,
        _ if header => return Err(invalid()),
        DataRef::Int(value) => value.to_string(),
        DataRef::Float(value) if value.is_finite() => value.to_string(),
        DataRef::Bool(value) => value.to_string(),
        DataRef::DateTimeIso(value) | DataRef::DurationIso(value) => checked_text(value, row, col)?,
        DataRef::DateTime(value)
            if value.is_datetime() && (0.0..2_958_466.0).contains(&value.as_f64()) =>
        {
            let (year, month, day, hour, minute, second, ms) = value.to_ymd_hms_milli();
            if hour == 0 && minute == 0 && second == 0 && ms == 0 {
                format!("{year:04}-{month:02}-{day:02}")
            } else {
                format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{ms:03}")
            }
        }
        _ => return Err(invalid()),
    };
    Ok(text)
}

fn checked_text(value: &str, row: u32, col: u32) -> Result<String, ImportError> {
    if value.len() > MAX_CELL_BYTES {
        return Err(ImportError::CellTooLarge {
            line: Some(u64::from(row + 1)),
            column: col as usize + 1,
        });
    }
    Ok(value.trim().to_owned())
}
