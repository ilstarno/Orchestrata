use crate::ReportDate;

pub struct AssignmentRow<'a> {
    pub work_item: &'a str,
    pub assigned_to: Option<&'a str>,
    pub scheduled_date: Option<ReportDate>,
    pub planned_minutes: Option<u32>,
}
