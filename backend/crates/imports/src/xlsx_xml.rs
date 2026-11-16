use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

use crate::{
    ImportError,
    constants::{MAX_COLUMNS, MAX_ROWS},
};

const MAX_DEPTH: usize = 48;
const MAX_NODES: usize = 350_000;
const MAX_ATTRIBUTES: usize = 64;
const MAX_ATTRIBUTE_BYTES: usize = 4_096;
const MAX_SHARED_STRINGS: usize = 16_384;

pub(crate) fn validate_xml(bytes: &[u8]) -> Result<(), ImportError> {
    if bytes.contains(&0) || std::str::from_utf8(bytes).is_err() {
        return Err(ImportError::InvalidWorkbook);
    }
    let mut reader = Reader::from_reader(bytes);
    let (mut depth, mut nodes, mut strings) = (0usize, 0usize, 0usize);
    loop {
        let event = reader
            .read_event()
            .map_err(|_| ImportError::InvalidWorkbook)?;
        match event {
            Event::Start(ref tag) | Event::Empty(ref tag) => {
                nodes += 1;
                if nodes > MAX_NODES {
                    return Err(ImportError::WorkbookExpandedTooLarge);
                }
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err(ImportError::WorkbookExpandedTooLarge);
                    }
                }
                if tag.local_name().as_ref() == "si" {
                    strings += 1;
                    if strings > MAX_SHARED_STRINGS {
                        return Err(ImportError::WorkbookExpandedTooLarge);
                    }
                }
                validate_tag(tag)?;
            }
            Event::End(_) => depth = depth.checked_sub(1).ok_or(ImportError::InvalidWorkbook)?,
            Event::DocType(_) | Event::PI(_) => {
                return Err(ImportError::UnsupportedWorkbookContent);
            }
            Event::Decl(decl) => {
                if let Some(encoding) = decl.encoding() {
                    let encoding = encoding.map_err(|_| ImportError::InvalidWorkbook)?;
                    if !encoding.eq_ignore_ascii_case("utf-8") {
                        return Err(ImportError::UnsupportedWorkbookContent);
                    }
                }
            }
            Event::Eof if depth == 0 => return Ok(()),
            Event::Eof => return Err(ImportError::InvalidWorkbook),
            _ => {}
        }
    }
}

fn validate_tag(tag: &BytesStart<'_>) -> Result<(), ImportError> {
    let local = tag.local_name();
    let local = local.as_ref();
    if local == "f" {
        return Err(ImportError::UnsupportedWorkbookContent);
    }
    for (index, attribute) in tag.attributes().enumerate() {
        if index >= MAX_ATTRIBUTES {
            return Err(ImportError::WorkbookExpandedTooLarge);
        }
        let attribute = attribute.map_err(|_| ImportError::InvalidWorkbook)?;
        if attribute.value.len() > MAX_ATTRIBUTE_BYTES {
            return Err(ImportError::WorkbookExpandedTooLarge);
        }
        let key = attribute.key.as_ref();
        let value = attribute.value.as_ref();
        match (local, key) {
            ("sst", "uniqueCount") => {
                if parse_usize(value.as_bytes())
                    .filter(|&n| n <= MAX_SHARED_STRINGS)
                    .is_none()
                {
                    return Err(ImportError::WorkbookExpandedTooLarge);
                }
            }
            ("dimension", "ref") => validate_dimension(value.as_bytes())?,
            ("c", "r") => {
                position(value.as_bytes()).ok_or(ImportError::WorkbookDimensions)?;
            }
            ("row", "r") => {
                if parse_usize(value.as_bytes())
                    .filter(|&r| (1..=MAX_ROWS + 1).contains(&r))
                    .is_none()
                {
                    return Err(ImportError::WorkbookDimensions);
                }
            }
            ("Relationship", "TargetMode") if !value.eq_ignore_ascii_case("internal") => {
                return Err(ImportError::UnsupportedWorkbookContent);
            }
            (_, "ContentType") if contains_ascii_ignore_case(value.as_bytes(), b"macroenabled") => {
                return Err(ImportError::UnsupportedWorkbookContent);
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_dimension(value: &[u8]) -> Result<(), ImportError> {
    let mut refs = value.split(|&byte| byte == b':');
    let first = refs
        .next()
        .and_then(position)
        .ok_or(ImportError::WorkbookDimensions)?;
    if let Some(last) = refs.next() {
        let last = position(last).ok_or(ImportError::WorkbookDimensions)?;
        if last.0 < first.0 || last.1 < first.1 || refs.next().is_some() {
            return Err(ImportError::WorkbookDimensions);
        }
    }
    Ok(())
}

fn position(value: &[u8]) -> Option<(usize, usize)> {
    let split = value.iter().position(u8::is_ascii_digit)?;
    if !(1..=2).contains(&split) {
        return None;
    }
    let col = value[..split].iter().try_fold(0usize, |col, letter| {
        let letter = letter.to_ascii_uppercase();
        if !letter.is_ascii_uppercase() {
            return None;
        }
        Some(col * 26 + usize::from(letter - b'A' + 1))
    })?;
    let row = parse_usize(&value[split..])?;
    if !(1..=MAX_COLUMNS).contains(&col) || !(1..=MAX_ROWS + 1).contains(&row) {
        return None;
    }
    Some((row, col))
}

fn parse_usize(value: &[u8]) -> Option<usize> {
    if value.is_empty() || value.len() > 8 || !value.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(value).ok()?.parse().ok()
}

fn contains_ascii_ignore_case(data: &[u8], value: &[u8]) -> bool {
    data.windows(value.len())
        .any(|part| part.eq_ignore_ascii_case(value))
}
