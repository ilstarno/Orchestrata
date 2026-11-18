use crate::ImportError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CsvStreamError<E> {
    Import(ImportError),
    Consumer(E),
}

impl<E> From<ImportError> for CsvStreamError<E> {
    fn from(error: ImportError) -> Self {
        Self::Import(error)
    }
}
