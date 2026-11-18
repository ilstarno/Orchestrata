use rust_xlsxwriter::{Worksheet, XlsxError};

use crate::{
    AssignmentReport,
    formats::{Formats, NAVY},
    render::{configure_sheet, merge_text},
};

const LAST_COL: u16 = 5;
const COLUMN_WIDTH: f64 = 15.0;

pub(crate) fn write(
    sheet: &mut Worksheet,
    report: &AssignmentReport<'_>,
    fmt: &Formats,
) -> Result<(), XlsxError> {
    sheet.set_name("Overview")?;
    configure_sheet(sheet, NAVY);
    sheet.set_portrait().set_print_fit_to_pages(1, 1);
    for col in 0..=LAST_COL {
        sheet.set_column_width(col, COLUMN_WIDTH)?;
    }
    sheet.set_print_area(0, 0, 10, LAST_COL)?;
    for (row, height) in [
        (0, 24),
        (1, 46),
        (3, 24),
        (5, 25),
        (6, 48),
        (9, 20),
        (10, 34),
    ] {
        sheet.set_row_height(row, height)?;
    }

    merge_text(
        sheet,
        0,
        0,
        LAST_COL,
        "ORCHESTRATA   /   ASSIGNMENT REPORT",
        &fmt.masthead,
    )?;
    merge_text(sheet, 1, 0, LAST_COL, report.title, &fmt.title)?;
    sheet.write_string_with_format(3, 0, "Plan date", &fmt.subtitle)?;
    sheet.write_datetime_with_format(3, 1, report.plan_date.to_excel()?, &fmt.date[0])?;

    let total = report.rows.len();
    let assigned = report
        .rows
        .iter()
        .filter(|row| row.assigned_to.is_some())
        .count();
    for (first_col, label, value) in [
        (0, "WORK ITEMS", total),
        (2, "ASSIGNED", assigned),
        (4, "UNASSIGNED", total - assigned),
    ] {
        merge_text(sheet, 5, first_col, first_col + 1, label, &fmt.metric_label)?;
        sheet.merge_range(6, first_col, 6, first_col + 1, "", &fmt.metric_value)?;
        sheet.write_number_with_format(6, first_col, value as f64, &fmt.metric_value)?;
    }

    merge_text(sheet, 9, 0, LAST_COL, "Read the plan", &fmt.header)?;
    merge_text(
        sheet,
        10,
        0,
        LAST_COL,
        "Open Assignments for the individual work items. Blank dates or durations were not supplied. Counts reflect this exported snapshot.",
        &fmt.subtitle,
    )?;
    Ok(())
}
