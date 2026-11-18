use std::{env, error::Error, fs};

use orchestrata_reports::{AssignmentReport, AssignmentRow, ReportDate, assignment_report_xlsx};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os()
        .nth(1)
        .ok_or("provide an output .xlsx path")?;
    let plan_date = ReportDate {
        year: 2026,
        month: 11,
        day: 25,
    };
    let rows = [
        AssignmentRow {
            work_item: "Visitor desk opening",
            assigned_to: Some("Staff A"),
            scheduled_date: Some(plan_date),
            planned_minutes: Some(90),
        },
        AssignmentRow {
            work_item: "Equipment inventory",
            assigned_to: Some("Staff B"),
            scheduled_date: Some(plan_date),
            planned_minutes: Some(45),
        },
        AssignmentRow {
            work_item: "Courier intake",
            assigned_to: None,
            scheduled_date: None,
            planned_minutes: Some(30),
        },
    ];
    let workbook = assignment_report_xlsx(&AssignmentReport {
        title: "Morning operations",
        plan_date,
        rows: &rows,
    })?;
    fs::write(path, workbook)?;
    Ok(())
}
