use std::io::{Cursor, Read};

use orchestrata_reports::{
    AssignmentReport, AssignmentRow, ReportDate, ReportError, assignment_report_xlsx,
};
use roxmltree::{Document, Node};
use zip::ZipArchive;

const PLAN_DATE: ReportDate = ReportDate {
    year: 2026,
    month: 11,
    day: 25,
};

#[test]
fn workbook_contains_styled_printable_sheets_and_native_numbers() {
    let rows = [
        AssignmentRow {
            work_item: "Inventory",
            assigned_to: Some("Staff A"),
            scheduled_date: Some(PLAN_DATE),
            planned_minutes: Some(90),
        },
        AssignmentRow {
            work_item: "Reception",
            assigned_to: None,
            scheduled_date: None,
            planned_minutes: None,
        },
    ];
    let mut zip = workbook("Morning operations", &rows);
    let index_xml = entry(&mut zip, "xl/workbook.xml");
    let index = Document::parse(&index_xml).unwrap();
    let sheets: Vec<_> = index
        .descendants()
        .filter(|n| n.has_tag_name("sheet"))
        .map(|n| n.attribute("name").unwrap())
        .collect();
    assert_eq!(sheets, ["Overview", "Assignments"]);
    let print_names: Vec<_> = index
        .descendants()
        .filter(|n| n.has_tag_name("definedName"))
        .map(|n| (n.attribute("name").unwrap(), n.text().unwrap()))
        .collect();
    assert!(print_names.contains(&("_xlnm.Print_Area", "Overview!$A$1:$F$11")));
    assert!(print_names.contains(&("_xlnm.Print_Area", "Assignments!$A$1:$E$6")));
    assert!(print_names.contains(&("_xlnm.Print_Titles", "Assignments!$4:$4")));

    let overview_xml = entry(&mut zip, "xl/worksheets/sheet1.xml");
    let overview = Document::parse(&overview_xml).unwrap();
    assert_eq!(number(cell(&overview, "A7")), "2");
    assert_eq!(number(cell(&overview, "C7")), "1");
    assert_eq!(number(cell(&overview, "E7")), "1");

    let detail_xml = entry(&mut zip, "xl/worksheets/sheet2.xml");
    let detail = Document::parse(&detail_xml).unwrap();
    assert_eq!(
        element(&detail, "pane").attribute("topLeftCell"),
        Some("B5")
    );
    assert_eq!(
        element(&detail, "autoFilter").attribute("ref"),
        Some("A4:E6")
    );
    assert_eq!(
        element(&detail, "pageSetup").attribute("orientation"),
        Some("landscape")
    );
    assert_eq!(
        element(&detail, "pageSetup")
            .attribute("fitToWidth")
            .unwrap_or("1"),
        "1"
    );
    assert_eq!(
        element(&detail, "pageSetUpPr").attribute("fitToPage"),
        Some("1")
    );
    assert_eq!(
        element(&detail, "pageSetup").attribute("fitToHeight"),
        Some("0")
    );
    assert_eq!(number(cell(&detail, "D5")), "46351");
    assert_eq!(number(cell(&detail, "E5")), "90");
    assert!(cell(&detail, "D6").first_element_child().is_none());
    assert!(cell(&detail, "D5").attribute("s").is_some());
    let styles = entry(&mut zip, "xl/styles.xml");
    assert!(styles.contains("yyyy-mm-dd"));
}

#[test]
fn user_supplied_formula_and_url_text_never_become_formulas_or_links() {
    let rows = [AssignmentRow {
        work_item: "=SUM(1,1)",
        assigned_to: Some("https://example.invalid"),
        scheduled_date: None,
        planned_minutes: None,
    }];
    let mut zip = workbook("+sample", &rows);
    let overview = entry(&mut zip, "xl/worksheets/sheet1.xml");
    let detail = entry(&mut zip, "xl/worksheets/sheet2.xml");
    let overview = Document::parse(&overview).unwrap();
    let detail = Document::parse(&detail).unwrap();
    assert_eq!(cell(&overview, "A2").attribute("t"), Some("s"));
    assert_eq!(cell(&detail, "A5").attribute("t"), Some("s"));
    assert_eq!(cell(&detail, "B5").attribute("t"), Some("s"));
    assert!(
        !detail
            .descendants()
            .any(|n| n.has_tag_name("f") || n.has_tag_name("hyperlink"))
    );
    let strings = entry(&mut zip, "xl/sharedStrings.xml");
    assert!(strings.contains("=SUM(1,1)"));
    assert!(strings.contains("https://example.invalid"));
    assert!(strings.contains("+sample"));
}

#[test]
fn empty_snapshots_and_validation_are_predictable() {
    let mut zip = workbook("Empty", &[]);
    let content = entry(&mut zip, "xl/worksheets/sheet2.xml");
    assert!(!content.contains("autoFilter"));
    assert!(entry(&mut zip, "xl/sharedStrings.xml").contains("No work items in this snapshot."));

    assert!(matches!(
        assignment_report_xlsx(&AssignmentReport {
            title: "Bad\nlabel",
            plan_date: PLAN_DATE,
            rows: &[]
        }),
        Err(ReportError::InvalidTitle),
    ));
    let invalid = [AssignmentRow {
        work_item: "Valid",
        assigned_to: Some(" "),
        scheduled_date: None,
        planned_minutes: None,
    }];
    assert!(matches!(
        assignment_report_xlsx(&AssignmentReport {
            title: "Valid",
            plan_date: PLAN_DATE,
            rows: &invalid
        }),
        Err(ReportError::InvalidLabel(1)),
    ));
}

fn workbook(title: &str, rows: &[AssignmentRow<'_>]) -> ZipArchive<Cursor<Vec<u8>>> {
    let bytes = assignment_report_xlsx(&AssignmentReport {
        title,
        plan_date: PLAN_DATE,
        rows,
    })
    .unwrap();
    ZipArchive::new(Cursor::new(bytes)).unwrap()
}

fn entry(zip: &mut ZipArchive<Cursor<Vec<u8>>>, path: &str) -> String {
    let mut xml = String::new();
    zip.by_name(path).unwrap().read_to_string(&mut xml).unwrap();
    xml
}

fn element<'doc, 'xml>(doc: &'doc Document<'xml>, tag: &str) -> Node<'doc, 'xml> {
    doc.descendants().find(|n| n.has_tag_name(tag)).unwrap()
}

fn cell<'doc, 'xml>(doc: &'doc Document<'xml>, coordinate: &str) -> Node<'doc, 'xml> {
    doc.descendants()
        .find(|n| n.has_tag_name("c") && n.attribute("r") == Some(coordinate))
        .unwrap()
}

fn number<'doc>(cell: Node<'doc, '_>) -> &'doc str {
    assert!(!matches!(cell.attribute("t"), Some("s" | "inlineStr")));
    let value = cell.first_element_child().unwrap();
    assert!(value.has_tag_name("v"));
    value.text().unwrap()
}
