use std::io::Read;

use crate::{
    CsvRow, CsvStreamError, ImportError, Mapping,
    constants::{MAX_STREAM_BYTES, MAX_STREAM_ROWS, STREAM_BATCH_ROWS},
    csv_import::{reader_from, visit_rows},
};

/// Reads an import of at most 16 MiB and 100,000 rows in batches of at most 128.
/// A callback borrows each normalized batch. Earlier callbacks may have run
/// even if a later record, input read, or callback fails. Stage the effects and
/// commit them only after this function returns success.
pub fn stream_csv_batches<R, F, E>(
    input: R,
    mapping: &Mapping,
    mut consume_batch: F,
) -> Result<usize, CsvStreamError<E>>
where
    R: Read,
    F: FnMut(&[CsvRow]) -> Result<(), E>,
{
    let mut reader = reader_from(input.take(MAX_STREAM_BYTES + 1));
    let mut batch = Vec::with_capacity(STREAM_BATCH_ROWS);
    let result = visit_rows(
        &mut reader,
        mapping,
        MAX_STREAM_ROWS,
        MAX_STREAM_BYTES,
        |row| {
            batch.push(row);
            if batch.len() == STREAM_BATCH_ROWS {
                consume_batch(&batch).map_err(CsvStreamError::Consumer)?;
                batch.clear();
            }
            Ok(())
        },
    );

    if matches!(result, Err(CsvStreamError::Consumer(_))) {
        return result;
    }
    if reader.get_ref().limit() == 0 {
        return Err(ImportError::SourceTooLarge.into());
    }
    let rows = result?;
    if !batch.is_empty() {
        consume_batch(&batch).map_err(CsvStreamError::Consumer)?;
    }
    Ok(rows)
}
