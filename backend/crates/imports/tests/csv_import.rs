use std::collections::BTreeMap;

use orchestrata_imports::{
    ColumnMapping, ImportError, Mapping, normalize_csv, suggest_csv_mapping,
};

const SAMPLE: &[u8] = b" Slot_ID ,Job Name,Count\r\n0007,\" Check, repair \",42\r\n";

fn job_mapping() -> Mapping {
    Mapping::new(vec![ColumnMapping::new("Job Name", "job-name")]).unwrap()
}

#[test]
fn exact_header_suggestions_handle_case_spacing_and_quotes() {
    let mapping = suggest_csv_mapping(SAMPLE, &["slot-id", "job-name"]).unwrap();
    assert_eq!(mapping.columns()[1].target(), "job-name");
    let output = normalize_csv(SAMPLE, &mapping).unwrap();
    assert_eq!(
        output,
        vec![BTreeMap::from([
            ("job-name".to_owned(), "Check, repair".to_owned()),
            ("slot-id".to_owned(), "0007".to_owned()),
        ])]
    );

    assert_eq!(
        suggest_csv_mapping(SAMPLE, &["jobname"]),
        Err(ImportError::MissingHeader("jobname".to_owned()))
    );
}

#[test]
fn versioned_mapping_can_be_serialized_and_reused_for_reordered_headers() {
    let mapping = Mapping::new(vec![
        ColumnMapping::new("Display Name", "job-name"),
        ColumnMapping::new("Ref ID", "external-id"),
    ])
    .unwrap();
    let json = serde_json::to_string(&mapping).unwrap();
    assert!(json.contains(r#""version":1"#));
    let loaded = serde_json::from_str::<Mapping>(&json).unwrap();
    let output = normalize_csv(b"REF-ID,display_name\n0041,Inspect lift\n", &loaded).unwrap();
    assert_eq!(output[0]["external-id"], "0041");
    assert_eq!(output[0]["job-name"], "Inspect lift");

    let unknown_version = json.replace(r#""version":1"#, r#""version":2"#);
    let loaded = serde_json::from_str::<Mapping>(&unknown_version).unwrap();
    assert_eq!(
        normalize_csv(SAMPLE, &loaded),
        Err(ImportError::UnsupportedMappingVersion(2))
    );

    let unknown_field = json.replacen("{", r#"{"surprise":true,"#, 1);
    assert!(serde_json::from_str::<Mapping>(&unknown_field).is_err());
}

#[test]
fn rejects_duplicate_columns_after_normalization() {
    let source = b"Job Name,job_name\nFirst,Second\n";
    assert_eq!(
        suggest_csv_mapping(source, &["job-name"]),
        Err(ImportError::DuplicateHeader("job-name".to_owned()))
    );
    assert_eq!(
        normalize_csv(source, &job_mapping()),
        Err(ImportError::DuplicateHeader("job-name".to_owned()))
    );
    let repeated_source = Mapping::new(vec![
        ColumnMapping::new("Job Name", "job-name"),
        ColumnMapping::new("JOB-NAME", "external-id"),
    ]);
    assert_eq!(
        repeated_source,
        Err(ImportError::DuplicateSource("job-name".to_owned()))
    );
    let repeated_target = Mapping::new(vec![
        ColumnMapping::new("Job Name", "job-name"),
        ColumnMapping::new("Different Name", "job-name"),
    ]);
    assert_eq!(
        repeated_target,
        Err(ImportError::DuplicateTarget("job-name".to_owned()))
    );
}

#[test]
fn rejects_missing_headers_record_shape_and_non_utf8() {
    assert_eq!(
        normalize_csv(b"", &job_mapping()),
        Err(ImportError::NoHeaders)
    );
    assert_eq!(
        normalize_csv(b"Different\nA\n", &job_mapping()),
        Err(ImportError::MissingHeader("job-name".to_owned()))
    );
    assert!(matches!(
        normalize_csv(b"Job Name,Other\nOnly one\n", &job_mapping()),
        Err(ImportError::InvalidCsv { line: Some(2) })
    ));
    assert!(matches!(
        normalize_csv(b"Job Name\n\xff\n", &job_mapping()),
        Err(ImportError::InvalidCsv { line: Some(2) })
    ));
}

#[test]
fn enforces_source_row_column_and_cell_budgets() {
    assert_eq!(
        normalize_csv(&vec![b'x'; 1_048_577], &job_mapping()),
        Err(ImportError::SourceTooLarge)
    );
    let wide = (0..65)
        .map(|i| format!("field-{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        suggest_csv_mapping(wide.as_bytes(), &["field-0"]),
        Err(ImportError::TooManyColumns)
    );
    let tall = format!("Job Name\n{}", "task\n".repeat(4_097));
    assert_eq!(
        normalize_csv(tall.as_bytes(), &job_mapping()),
        Err(ImportError::TooManyRows)
    );
    let long = format!("Job Name\n{}", "a".repeat(4_097));
    assert_eq!(
        normalize_csv(long.as_bytes(), &job_mapping()),
        Err(ImportError::CellTooLarge {
            line: Some(2),
            column: 1,
        })
    );
}
