//! A UKI is tens of megabytes, so the helper reads only its header and
//! the few small sections the Boot page shows.

#[cfg(test)]
mod tests;

/// Every UKI's section table ends within its first kilobyte.
pub const HEADER_BYTES: usize = 4096;

const PE_OFFSET_AT: usize = 0x3c;
const COFF_BYTES: usize = 20;
const SECTION_BYTES: usize = 40;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

pub fn sections(header: &[u8]) -> Vec<Section> {
    if header.get(..2) != Some(b"MZ") {
        return Vec::new();
    }

    let Some(pe) = u32_at(header, PE_OFFSET_AT).and_then(|at| usize::try_from(at).ok()) else {
        return Vec::new();
    };

    if header.get(pe..pe.saturating_add(4)) != Some(b"PE\0\0") {
        return Vec::new();
    }

    let coff = pe.saturating_add(4);
    let count = u16_at(header, coff.saturating_add(2)).unwrap_or(0);
    let optional = u16_at(header, coff.saturating_add(16)).unwrap_or(0);
    let table = coff
        .saturating_add(COFF_BYTES)
        .saturating_add(usize::from(optional));

    (0..usize::from(count))
        .map_while(|index| {
            section(
                header,
                table.saturating_add(index.saturating_mul(SECTION_BYTES)),
            )
        })
        .collect()
}

fn section(header: &[u8], at: usize) -> Option<Section> {
    let name = header.get(at..at.checked_add(8)?)?;
    let name = String::from_utf8_lossy(name)
        .trim_end_matches('\0')
        .to_owned();

    let virtual_size = u32_at(header, at.checked_add(8)?)?;

    let raw_size = u32_at(header, at.checked_add(16)?)?;
    let offset = u32_at(header, at.checked_add(20)?)?;

    // The raw size is padded to the file alignment, the virtual size is not.
    Some(Section {
        name,
        offset,
        size: virtual_size.min(raw_size),
    })
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let field = bytes.get(at..at.checked_add(2)?)?;

    Some(u16::from_le_bytes(field.try_into().ok()?))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let field = bytes.get(at..at.checked_add(4)?)?;

    Some(u32::from_le_bytes(field.try_into().ok()?))
}
