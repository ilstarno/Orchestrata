use rust_xlsxwriter::{Format, Workbook, Worksheet, XlsxError};

use crate::{AssignmentReport, ReportError, assignments_tab, formats::Formats, overview_tab};

const MAX_REPORT_ROWS: usize = 10_000;
const MAX_TITLE_CHARS: usize = 96;
const MAX_LABEL_CHARS: usize = 120;
const A4_PAPER: u8 = 9;

/// Writes a two-sheet XLSX snapshot to memory. All caller-supplied text is written as literal
/// strings, never as formulas or hyperlinks. This does not save or transmit the workbook.
pub fn assignment_report_xlsx(report: &AssignmentReport<'_>) -> Result<Vec<u8>, ReportError> {
    validate(report)?;
    let formats = Formats::new();
    let mut workbook = Workbook::new();
    overview_tab::write(workbook.add_worksheet(), report, &formats)?;
    assignments_tab::write(workbook.add_worksheet(), report, &formats)?;
    Ok(workbook.save_to_buffer()?)
}

fn validate(report: &AssignmentReport<'_>) -> Result<(), ReportError> {
    if report.rows.len() > MAX_REPORT_ROWS {
        return Err(ReportError::TooManyRows);
    }
    if !valid_label(report.title, MAX_TITLE_CHARS) {
        return Err(ReportError::InvalidTitle);
    }
    report
        .plan_date
        .to_excel()
        .map_err(|_| ReportError::InvalidPlanDate)?;
    for (index, row) in report.rows.iter().enumerate() {
        if !valid_label(row.work_item, MAX_LABEL_CHARS)
            || row
                .assigned_to
                .is_some_and(|name| !valid_label(name, MAX_LABEL_CHARS))
        {
            return Err(ReportError::InvalidLabel(index + 1));
        }
        if let Some(date) = row.scheduled_date {
            date.to_excel()
                .map_err(|_| ReportError::InvalidScheduledDate(index + 1))?;
        }
    }
    Ok(())
}

fn valid_label(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().take(max_chars + 1).count() <= max_chars
        && !value.chars().any(char::is_control)
}

pub(crate) fn configure_sheet(sheet: &mut Worksheet, color: u32) {
    sheet
        .set_screen_gridlines(false)
        .set_tab_color(color)
        .set_paper_size(A4_PAPER)
        .set_margins(0.35, 0.35, 0.42, 0.42, 0.2, 0.2)
        .set_footer("&LOrchestrata assignment snapshot&RPage &P of &N");
}

pub(crate) fn merge_text(
    sheet: &mut Worksheet,
    row: u32,
    first_col: u16,
    last_col: u16,
    text: &str,
    format: &Format,
) -> Result<(), XlsxError> {
    sheet.merge_range(row, first_col, row, last_col, "", format)?;
    sheet.write_string_with_format(row, first_col, text, format)?;
    Ok(())
}
