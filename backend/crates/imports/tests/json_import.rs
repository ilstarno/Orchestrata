use orchestrata_imports::{
    ColumnMapping, ImportError, JsonScalar, Mapping, normalize_json_records,
};

fn mapping() -> Mapping {
    Mapping::new(vec![
        ColumnMapping::new("Source Key", "id"),
        ColumnMapping::new("Enabled", "active"),
    ])
    .unwrap()
}

#[test]
fn maps_flat_records_without_coercing_scalar_values() {
    let rows = normalize_json_records(
        br#"[{"source_key":"0007", "ENABLED":true, "ignored":null},
             {"Enabled":null, "SOURCE-KEY":-9223372036854775808}]"#,
        &mapping(),
    )
    .unwrap();
    assert_eq!(rows[0]["id"], JsonScalar::Text("0007".to_owned()));
    assert_eq!(rows[0]["active"], JsonScalar::Boolean(true));
    assert_eq!(rows[1]["id"], JsonScalar::Integer(i64::MIN));
    assert_eq!(rows[1]["active"], JsonScalar::Null);
}

#[test]
fn rejects_duplicate_conflicting_names_and_missing_mapped_fields() {
    for source in [
        br#"[{"Source Key":1,"source_key":2,"Enabled":true}]"#.as_slice(),
        br#"[{"Source Key":1,"Source Key":2,"Enabled":true}]"#.as_slice(),
    ] {
        assert_eq!(
            normalize_json_records(source, &mapping()),
            Err(ImportError::DuplicateHeader("source-key".to_owned()))
        );
    }
    assert_eq!(
        normalize_json_records(br#"[{"Source Key":2}]"#, &mapping()),
        Err(ImportError::MissingHeader("enabled".to_owned()))
    );
}

#[test]
fn rejects_unsupported_shapes_types_and_trailing_data() {
    for source in [
        br#"{"Source Key":1,"Enabled":true}"#.as_slice(),
        br#"[{"Source Key":1.5,"Enabled":true}]"#.as_slice(),
        br#"[{"Source Key":9223372036854775808,"Enabled":true}]"#.as_slice(),
        br#"[{"Source Key":{"key":1},"Enabled":true}]"#.as_slice(),
        br#"[{"Source Key":1,"Enabled":[true]}]"#.as_slice(),
        br#"[] true"#.as_slice(),
    ] {
        assert!(matches!(
            normalize_json_records(source, &mapping()),
            Err(ImportError::InvalidJson { .. })
        ));
    }
}

#[test]
fn bounds_input_bytes_rows_and_text_cells() {
    assert_eq!(
        normalize_json_records(&vec![b' '; 1_048_577], &mapping()),
        Err(ImportError::SourceTooLarge)
    );
    let long = format!(
        r#"[{{"Source Key":"{}","Enabled":true}}]"#,
        "a".repeat(4_097)
    );
    assert_eq!(
        normalize_json_records(long.as_bytes(), &mapping()),
        Err(ImportError::CellTooLarge {
            line: None,
            column: 1
        })
    );
    let tall = format!(
        "[{}]",
        vec![r#"{"Source Key":1,"Enabled":true}"#; 4_097].join(",")
    );
    assert!(matches!(
        normalize_json_records(tall.as_bytes(), &mapping()),
        Err(ImportError::InvalidJson { .. })
    ));
}
