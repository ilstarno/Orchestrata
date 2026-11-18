use rust_xlsxwriter::XlsxError;

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("report title must contain 1 to 96 printable characters")]
    InvalidTitle,
    #[error("assignment report exceeds the 10,000-row export limit")]
    TooManyRows,
    #[error("invalid work item or assigned-to label in assignment {0}")]
    InvalidLabel(usize),
    #[error("invalid plan date")]
    InvalidPlanDate,
    #[error("invalid scheduled date in assignment {0}")]
    InvalidScheduledDate(usize),
    #[error("could not build Excel workbook")]
    Workbook(#[from] XlsxError),
}
