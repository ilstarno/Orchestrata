use crate::{ImportError, constants::MAX_HEADER_BYTES};

pub(crate) fn normalized_header(header: &str) -> Result<String, ImportError> {
    if header.trim().is_empty() || header.len() > MAX_HEADER_BYTES {
        return Err(ImportError::InvalidHeader);
    }
    if header
        .chars()
        .any(|ch| ch.is_control() && !ch.is_whitespace())
    {
        return Err(ImportError::InvalidHeader);
    }
    let header = header
        .split(|ch: char| ch.is_whitespace() || ch == '_' || ch == '-')
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("-");
    if header.is_empty() || header.len() > MAX_HEADER_BYTES {
        return Err(ImportError::InvalidHeader);
    }
    Ok(header)
}

pub(crate) fn validate_target(target: &str) -> Result<(), ImportError> {
    if target.len() > MAX_HEADER_BYTES
        || !target
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_lowercase)
        || target.split('-').any(|part| part.is_empty())
        || !target
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ImportError::InvalidTarget);
    }
    Ok(())
}
