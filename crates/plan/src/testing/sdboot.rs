use crate::sdboot::{LoaderEntry, sections};

pub const GARUDA_7_3_1_1_GARUDA_EFI_HEAD: &[u8] =
    include_bytes!("../sdboot/fixtures/garuda-7.3.1-1-garuda.efi.head");
pub const GARUDA_7_3_1_1_GARUDA_FALLBACK_EFI_HEAD: &[u8] =
    include_bytes!("../sdboot/fixtures/garuda-7.3.1-1-garuda-fallback.efi.head");
pub const LOADER_CONF: &str = include_str!("../sdboot/fixtures/loader.conf");
pub const MOUNTINFO: &str = include_str!("../sdboot/fixtures/mountinfo");
pub const BARE_METAL_BOOTCTL_LIST: &str =
    include_str!("../sdboot/fixtures/bare-metal/bootctl-list.json");
pub const IMAGE_BOOTCTL_LIST: &str = include_str!("../sdboot/fixtures/image/bootctl-list.json");
pub const SNAPSHOTS_BOOTCTL_LIST: &str =
    include_str!("../sdboot/fixtures/snapshots/bootctl-list.json");

/// The text of the section `name` of a UKI whose first bytes are `image`.
///
/// # Panics
///
/// When `image` has no such section or it is no UTF-8.
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test helper: a capture without the section fails the test"
)]
pub fn section(image: &[u8], name: &str) -> String {
    let found = sections(image)
        .into_iter()
        .find(|section| section.name == name)
        .unwrap();

    let start = usize::try_from(found.offset).unwrap();
    let end = start + usize::try_from(found.size).unwrap();

    String::from_utf8(image[start..end].to_vec()).unwrap()
}

/// The entry of the UKI `file` whose first bytes are `image`.
///
/// # Panics
///
/// When `image` lacks `.osrel` or `.uname`, or they name no entry.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a capture that names no entry fails the test"
)]
pub fn uki(file: &str, image: &[u8]) -> LoaderEntry {
    LoaderEntry::uki(
        file,
        &section(image, ".osrel"),
        Some(&section(image, ".uname")),
    )
    .unwrap()
}
