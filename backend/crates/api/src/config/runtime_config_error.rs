use std::fmt::{self, Display, Formatter};

use super::runtime_config::HTTP_BIND_ENV;

#[derive(Debug)]
pub enum RuntimeConfigError {
    InvalidHttpBind,
    NonUnicodeHttpBind,
}

impl Display for RuntimeConfigError {
    fn fmt(&self, output: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHttpBind => write!(
                output,
                "{HTTP_BIND_ENV} must be an IP address and port (e.g. 127.0.0.1:3000 or [::]:8080)"
            ),
            Self::NonUnicodeHttpBind => write!(output, "{HTTP_BIND_ENV} must be valid Unicode"),
        }
    }
}

impl std::error::Error for RuntimeConfigError {}
