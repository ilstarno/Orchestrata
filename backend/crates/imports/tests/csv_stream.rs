use std::{
    convert::Infallible,
    io::{self, Read},
};

use orchestrata_imports::{
    ColumnMapping, CsvStreamError, ImportError, Mapping, normalize_csv, stream_csv_batches,
};

const SOURCE_HEADER: &[u8] = b"Stop ID,Display Name\n";

fn mapping() -> Mapping {
    Mapping::new(vec![
        ColumnMapping::new("display_name", "name"),
        ColumnMapping::new("stop-id", "id"),
    ])
    .unwrap()
}

#[test]
fn streams_the_city_workload_without_raising_the_preview_limit() {
    let mut source = SOURCE_HEADER.to_vec();
    for stop in 1..=16_030 {
        source.extend_from_slice(format!("{stop:05}, Stop {stop} \n").as_bytes());
    }
    assert_eq!(
        normalize_csv(&source, &mapping()),
        Err(ImportError::TooManyRows)
    );

    let mut visited = 0;
    let mut last_batch = 0;
    let count = stream_csv_batches(source.as_slice(), &mapping(), |batch| {
        assert!((1..=128).contains(&batch.len()));
        for row in batch {
            visited += 1;
            assert_eq!(row["id"], format!("{visited:05}"));
            assert_eq!(row["name"], format!("Stop {visited}"));
        }
        last_batch = batch.len();
        Ok::<(), Infallible>(())
    })
    .unwrap();
    assert_eq!((count, visited, last_batch), (16_030, 16_030, 30));
}

#[test]
fn reports_late_errors_after_staged_batches_without_emitting_a_partial_tail() {
    let mut source = SOURCE_HEADER.to_vec();
    for _ in 0..145 {
        source.extend_from_slice(b"00001, Dock\n");
    }
    source.extend_from_slice(b"missing second column\n");
    let mut staged_rows = 0;
    let result = stream_csv_batches(source.as_slice(), &mapping(), |batch| {
        staged_rows += batch.len();
        Ok::<(), Infallible>(())
    });
    assert_eq!(staged_rows, 128);
    assert_eq!(
        result,
        Err(CsvStreamError::Import(ImportError::InvalidCsv {
            line: Some(147)
        }))
    );
}

#[test]
fn reports_consumer_failure_and_stops_delivery() {
    let mut source = SOURCE_HEADER.to_vec();
    for _ in 0..256 {
        source.extend_from_slice(b"00001, Dock\n");
    }
    let mut callback_count = 0;
    let result = stream_csv_batches(source.as_slice(), &mapping(), |batch| {
        callback_count += 1;
        assert_eq!(batch.len(), 128);
        Err("staging sink failed")
    });
    assert_eq!(result, Err(CsvStreamError::Consumer("staging sink failed")));
    assert_eq!(callback_count, 1);
}

#[test]
fn limits_input_bytes_even_when_excess_input_is_blank() {
    let budget = 16 * 1_048_576;
    for extra in [0, 1] {
        let source = SOURCE_HEADER
            .chain(io::repeat(b'\n').take((budget - SOURCE_HEADER.len() + extra) as u64));
        let result = stream_csv_batches(source, &mapping(), |_| -> Result<(), Infallible> {
            panic!("blank rows must not emit a batch")
        });
        if extra == 0 {
            assert_eq!(result, Ok(0));
        } else {
            assert_eq!(
                result,
                Err(CsvStreamError::Import(ImportError::SourceTooLarge))
            );
        }
    }
}

#[test]
fn rejects_more_than_the_streaming_row_budget() {
    let mut source = SOURCE_HEADER.to_vec();
    for _ in 0..100_001 {
        source.extend_from_slice(b"1,A\n");
    }
    let result = stream_csv_batches(source.as_slice(), &mapping(), |_| Ok::<(), Infallible>(()));
    assert_eq!(
        result,
        Err(CsvStreamError::Import(ImportError::TooManyRows))
    );
}
