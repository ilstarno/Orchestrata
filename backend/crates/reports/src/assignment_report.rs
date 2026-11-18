use crate::{AssignmentRow, ReportDate};

/// An already prepared planning snapshot. This report does not run a solver.
pub struct AssignmentReport<'a> {
    pub title: &'a str,
    pub plan_date: ReportDate,
    pub rows: &'a [AssignmentRow<'a>],
}
