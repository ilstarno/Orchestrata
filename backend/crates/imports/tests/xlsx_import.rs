use std::io::{Cursor, Read, Write};

use orchestrata_imports::{ColumnMapping, ImportError, Mapping, normalize_xlsx};
use rust_xlsxwriter::{ExcelDateTime, Format, Workbook};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

#[test]
fn named_sheet_uses_first_row_and_decodes_only_explicitly_mapped_cells() {
    let data = synthetic();
    let mapping = mapping(&[
        ("work item", "job"),
        ("estimated minutes", "minutes"),
        ("plan date", "date"),
        ("enabled", "active"),
    ]);
    let rows = normalize_xlsx(&data, "Schedule", &mapping).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["job"], "=literal text");
    assert_eq!(rows[0]["minutes"], "90");
    assert_eq!(rows[0]["date"], "2026-11-25");
    assert_eq!(rows[0]["active"], "true");
    assert_eq!(rows[1]["job"], "Courier intake");
    assert_eq!(rows[1]["minutes"], "");
    assert_eq!(
        normalize_xlsx(&data, "schedule", &mapping),
        Err(ImportError::WorksheetNotFound)
    );
    assert_eq!(
        normalize_xlsx(&data, "Ignored", &mapping),
        Err(ImportError::NoHeaders)
    );
}

#[test]
fn formula_cells_and_binary_package_parts_are_rejected() {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Schedule").unwrap();
    sheet.write_string(0, 0, "Work item").unwrap();
    sheet.write_formula(1, 0, "=1+1").unwrap();
    let data = workbook.save_to_buffer().unwrap();
    let mapping = mapping(&[("Work item", "job")]);
    assert_eq!(
        normalize_xlsx(&data, "Schedule", &mapping),
        Err(ImportError::UnsupportedWorkbookContent)
    );
    let data = rewrite_package(&synthetic(), |_, _| {}, Some(("xl/extra.bin", b"stub")));
    assert_eq!(
        normalize_xlsx(&data, "Schedule", &mapping),
        Err(ImportError::UnsupportedWorkbookContent)
    );
}

#[test]
fn huge_or_reversed_dimensions_and_shared_string_hints_fail_preflight() {
    let mapping = mapping(&[("Work item", "job")]);
    for shape in ["A1:B99999", "B5:A1"] {
        let data = rewrite_package(
            &synthetic(),
            |part, bytes| {
                if part == "xl/worksheets/sheet2.xml" {
                    replace_attribute(bytes, "dimension", "ref", shape);
                }
            },
            None,
        );
        assert_eq!(
            normalize_xlsx(&data, "Schedule", &mapping),
            Err(ImportError::WorkbookDimensions)
        );
    }
    let data = rewrite_package(
        &synthetic(),
        |part, bytes| {
            if part == "xl/sharedStrings.xml" {
                replace_attribute(bytes, "sst", "uniqueCount", "99999999");
            }
        },
        None,
    );
    assert_eq!(
        normalize_xlsx(&data, "Schedule", &mapping),
        Err(ImportError::WorkbookExpandedTooLarge)
    );
}

#[test]
fn expanded_zip_members_and_retained_shared_text_are_separately_limited() {
    let mapping = mapping(&[("Work item", "job")]);
    let mut huge_part = vec![b' '; 8 * 1024 * 1024 + 1];
    huge_part[0] = b'<';
    let data = rewrite_package(&synthetic(), |_, _| {}, Some(("xl/large.xml", &huge_part)));
    assert!(data.len() < 1_048_576);
    assert_eq!(
        normalize_xlsx(&data, "Schedule", &mapping),
        Err(ImportError::WorkbookExpandedTooLarge)
    );

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Schedule").unwrap();
    sheet.write_string(0, 0, "Work item").unwrap();
    let repeated = "x".repeat(4_096);
    for row in 1..=1025 {
        sheet.write_string(row, 0, &repeated).unwrap();
    }
    let data = workbook.save_to_buffer().unwrap();
    assert!(data.len() < 1_048_576);
    assert_eq!(
        normalize_xlsx(&data, "Schedule", &mapping),
        Err(ImportError::WorkbookOutputTooLarge)
    );
}

#[test]
fn missing_duplicate_and_nontext_headers_are_rejected() {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Schedule").unwrap();
    sheet.write_string(0, 0, "Work item").unwrap();
    sheet.write_string(0, 1, "WORK_ITEM").unwrap();
    let data = workbook.save_to_buffer().unwrap();
    assert_eq!(
        normalize_xlsx(&data, "Schedule", &mapping(&[("Work item", "job")])),
        Err(ImportError::DuplicateWorkbookHeader)
    );

    let sheet = workbook.worksheet_from_name("Schedule").unwrap();
    sheet.write_number(0, 1, 42).unwrap();
    let data = workbook.save_to_buffer().unwrap();
    assert_eq!(
        normalize_xlsx(&data, "Schedule", &mapping(&[("Work item", "job")])),
        Err(ImportError::UnsupportedWorkbookCell { row: 1, column: 2 })
    );
    assert_eq!(
        normalize_xlsx(&synthetic(), "Schedule", &mapping(&[("Unknown", "job")])),
        Err(ImportError::MissingWorkbookHeader)
    );
}

fn synthetic() -> Vec<u8> {
    let mut workbook = Workbook::new();
    workbook.add_worksheet().set_name("Ignored").unwrap();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Schedule").unwrap();
    for (column, header) in ["Work item", "Estimated minutes", "Plan Date", "Enabled"]
        .iter()
        .enumerate()
    {
        sheet.write_string(0, column as u16, *header).unwrap();
    }
    sheet.write_string(1, 0, "=literal text").unwrap();
    sheet.write_number(1, 1, 90).unwrap();
    sheet
        .write_datetime_with_format(
            1,
            2,
            ExcelDateTime::from_ymd(2026, 11, 25).unwrap(),
            &Format::new().set_num_format("yyyy-mm-dd"),
        )
        .unwrap();
    sheet.write_boolean(1, 3, true).unwrap();
    sheet.write_string(3, 0, "Courier intake").unwrap();
    workbook.save_to_buffer().unwrap()
}

fn mapping(pairs: &[(&str, &str)]) -> Mapping {
    Mapping::new(
        pairs
            .iter()
            .map(|&(source, target)| ColumnMapping::new(source, target))
            .collect(),
    )
    .unwrap()
}

fn rewrite_package(
    data: &[u8],
    mut transform: impl FnMut(&str, &mut Vec<u8>),
    extra: Option<(&str, &[u8])>,
) -> Vec<u8> {
    let mut archive = ZipArchive::new(Cursor::new(data)).unwrap();
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        let name = entry.name().unwrap();
        transform(name.as_ref(), &mut bytes);
        output.start_file(name.as_ref(), options).unwrap();
        output.write_all(&bytes).unwrap();
    }
    if let Some((name, data)) = extra {
        output.start_file(name, options).unwrap();
        output.write_all(data).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn replace_attribute(bytes: &mut Vec<u8>, tag: &str, name: &str, replacement: &str) {
    let mut source = String::from_utf8(std::mem::take(bytes)).unwrap();
    let tag_start = source.find(&format!("<{tag}")).unwrap();
    let attribute = format!("{name}=\"");
    let start = tag_start + source[tag_start..].find(&attribute).unwrap() + attribute.len();
    let end = start + source[start..].find('"').unwrap();
    source.replace_range(start..end, replacement);
    *bytes = source.into_bytes();
}
