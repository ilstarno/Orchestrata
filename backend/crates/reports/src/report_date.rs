use rust_xlsxwriter::{ExcelDateTime, XlsxError};

#[derive(Clone, Copy)]
pub struct ReportDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl ReportDate {
    pub(crate) fn to_excel(self) -> Result<ExcelDateTime, XlsxError> {
        if (self.year, self.month, self.day) == (1900, 2, 29) || self.year < 1900 {
            return Err(XlsxError::DateTimeRangeError(
                "expected a valid calendar date starting in 1900".into(),
            ));
        }
        ExcelDateTime::from_ymd(self.year, self.month, self.day)
    }
}
