use std::{
    collections::HashSet,
    io::{Cursor, Read},
};

use zip::{CompressionMethod, ZipArchive};

use crate::{ImportError, constants::MAX_BYTES, xlsx_xml::validate_xml};

const EOCD_BYTES: usize = 22;
const MAX_PARTS: usize = 128;
const MAX_PART_BYTES: u64 = 8 * 1024 * 1024;
const MAX_EXPANDED_BYTES: u64 = 16 * 1024 * 1024;

// Accept only a classic single-disk archive. Bound directory entries before constructing a ZIP parser.
fn directory_count(data: &[u8]) -> Result<usize, ImportError> {
    let end = data
        .len()
        .checked_sub(EOCD_BYTES)
        .ok_or(ImportError::InvalidWorkbook)?;
    let footer = &data[end..];
    if &footer[..4] != b"PK\x05\x06"
        || footer[4..8] != [0; 4]
        || footer[20..22] != [0; 2]
        || end
            .checked_sub(20)
            .is_some_and(|p| data[p..].starts_with(b"PK\x06\x07"))
    {
        return Err(ImportError::InvalidWorkbook);
    }
    let count = usize::from(u16::from_le_bytes([footer[10], footer[11]]));
    let directory_bytes = u32::from_le_bytes(footer[12..16].try_into().unwrap()) as usize;
    let directory_start = u32::from_le_bytes(footer[16..20].try_into().unwrap()) as usize;
    if count == 0
        || count > MAX_PARTS
        || footer[8..10] != footer[10..12]
        || directory_start.checked_add(directory_bytes) != Some(end)
        || directory_bytes < 46 * count
    {
        return Err(ImportError::InvalidWorkbook);
    }
    Ok(count)
}

pub(crate) fn validate_archive(data: &[u8]) -> Result<(), ImportError> {
    if data.len() > MAX_BYTES {
        return Err(ImportError::SourceTooLarge);
    }
    let count = directory_count(data)?;
    let mut archive =
        ZipArchive::new(Cursor::new(data)).map_err(|_| ImportError::InvalidWorkbook)?;
    if archive.len() != count
        || archive
            .has_overlapping_files()
            .map_err(|_| ImportError::InvalidWorkbook)?
    {
        return Err(ImportError::InvalidWorkbook);
    }
    let mut names = HashSet::with_capacity(count);
    let mut expanded_bytes = 0;
    let mut xml = Vec::new();
    for index in 0..count {
        let entry = archive
            .by_index(index)
            .map_err(|_| ImportError::InvalidWorkbook)?;
        let raw_name = entry.name().map_err(|_| ImportError::InvalidWorkbook)?;
        let name = raw_name.to_ascii_lowercase();
        if raw_name.len() > 256
            || !raw_name.is_ascii()
            || entry.enclosed_name().is_none()
            || !names.insert(name.clone())
        {
            return Err(ImportError::InvalidWorkbook);
        }
        if entry.is_dir() {
            if entry.size() != 0 {
                return Err(ImportError::InvalidWorkbook);
            }
            continue;
        }
        if !(name.ends_with(".xml") || name.ends_with(".rels"))
            || !matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err(ImportError::UnsupportedWorkbookContent);
        }
        let limit = MAX_PART_BYTES.min(MAX_EXPANDED_BYTES - expanded_bytes);
        let expected = entry.size();
        if expected > limit {
            return Err(ImportError::WorkbookExpandedTooLarge);
        }
        xml.clear();
        entry
            .take(limit + 1)
            .read_to_end(&mut xml)
            .map_err(|_| ImportError::InvalidWorkbook)?;
        let actual = xml.len() as u64;
        if actual > limit {
            return Err(ImportError::WorkbookExpandedTooLarge);
        }
        if actual != expected {
            return Err(ImportError::InvalidWorkbook);
        }
        expanded_bytes += actual;
        validate_xml(&xml)?;
    }
    Ok(())
}
