#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("identifier must be uuidv7")]
    InvalidIdentifier,
    #[error("score arithmetic overflow")]
    ScoreOverflow,
}
