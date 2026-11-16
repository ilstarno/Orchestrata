#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    SourceTooLarge,
    InvalidCsv { line: Option<u64> },
    InvalidJson { line: usize },
    NoHeaders,
    TooManyColumns,
    TooManyRows,
    InvalidHeader,
    DuplicateHeader(String),
    MissingHeader(String),
    InvalidTarget,
    UnsupportedMappingVersion(u16),
    InvalidMappingCount,
    DuplicateSource(String),
    DuplicateTarget(String),
    CellTooLarge { line: Option<u64>, column: usize },
}
