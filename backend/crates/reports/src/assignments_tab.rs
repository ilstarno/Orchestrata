use rust_xlsxwriter::{Worksheet, XlsxError};

use crate::{
    AssignmentReport,
    formats::{Formats, TEAL},
    render::{configure_sheet, merge_text},
};

const HEADER_ROW: u32 = 3;
const FIRST_DATA_ROW: u32 = HEADER_ROW + 1;
const LAST_COL: u16 = 4;
const HEADERS: [(f64, &str); 5] = [
    (36.0, "Work item"),
    (26.0, "Assigned to"),
    (16.0, "Status"),
    (18.0, "Scheduled date"),
    (18.0, "Planned minutes"),
];

pub(crate) fn write(
    sheet: &mut Worksheet,
    report: &AssignmentReport<'_>,
    fmt: &Formats,
) -> Result<(), XlsxError> {
    sheet.set_name("Assignments")?;
    configure_sheet(sheet, TEAL);
    sheet.set_landscape().set_print_fit_to_pages(1, 0);
    let last_row = HEADER_ROW + report.rows.len().max(1) as u32;
    sheet.set_print_area(0, 0, last_row, LAST_COL)?;
    sheet.set_repeat_rows(HEADER_ROW, HEADER_ROW)?;
    sheet.set_freeze_panes(FIRST_DATA_ROW, 1)?;
    sheet.set_row_height(0, 44)?;
    sheet.set_row_height(1, 28)?;
    sheet.set_row_height(HEADER_ROW, 26)?;
    merge_text(sheet, 0, 0, LAST_COL, "Assignments", &fmt.title)?;
    merge_text(
        sheet,
        1,
        0,
        LAST_COL,
        if report.rows.is_empty() {
            "The Overview shows the totals for this export."
        } else {
            "Use the filters to review individual work items, owners, and unassigned work."
        },
        &fmt.subtitle,
    )?;
    for (col, (width, label)) in HEADERS.iter().enumerate() {
        sheet.set_column_width(col as u16, *width)?;
        sheet.write_string_with_format(HEADER_ROW, col as u16, *label, &fmt.header)?;
    }

    if report.rows.is_empty() {
        sheet.set_row_height(FIRST_DATA_ROW, 26)?;
        return merge_text(
            sheet,
            FIRST_DATA_ROW,
            0,
            LAST_COL,
            "No work items in this snapshot.",
            &fmt.subtitle,
        );
    }
    sheet.autofilter(HEADER_ROW, 0, last_row, LAST_COL)?;

    for (index, assignment) in report.rows.iter().enumerate() {
        let row = FIRST_DATA_ROW + index as u32;
        let stripe = index % fmt.data.len();
        let data = &fmt.data[stripe];
        let lines = text_lines(assignment.work_item, 30).max(
            assignment
                .assigned_to
                .map_or(1, |value| text_lines(value, 20)),
        );
        sheet.set_row_height(row, (15 * lines + 10) as f64)?;
        sheet.write_string_with_format(row, 0, assignment.work_item, data)?;
        if let Some(resource) = assignment.assigned_to {
            sheet.write_string_with_format(row, 1, resource, data)?;
            sheet.write_string_with_format(row, 2, "Assigned", &fmt.status[1])?;
        } else {
            sheet.write_blank(row, 1, data)?;
            sheet.write_string_with_format(row, 2, "Unassigned", &fmt.status[0])?;
        }
        if let Some(date) = assignment.scheduled_date {
            sheet.write_datetime_with_format(row, 3, date.to_excel()?, &fmt.date[stripe])?;
        } else {
            sheet.write_blank(row, 3, data)?;
        }
        if let Some(minutes) = assignment.planned_minutes {
            sheet.write_number_with_format(row, 4, f64::from(minutes), &fmt.number[stripe])?;
        } else {
            sheet.write_blank(row, 4, data)?;
        }
    }
    Ok(())
}

fn text_lines(text: &str, width: usize) -> usize {
    text.chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum::<usize>()
        .div_ceil(width)
}
