use super::{EntryKind, LoaderEntry, entry_id};
use crate::sdboot::{LoaderSettings, SdBootMenu, sections};
use crate::{KernelRelease, LoaderEntryId};

/// The first bytes of the Garuda Lima image's UKIs, up to the end of
/// `.uname`.
const UKI: &[u8] = include_bytes!("../fixtures/garuda-7.3.1-1-garuda.efi.head");
const FALLBACK: &[u8] = include_bytes!("../fixtures/garuda-7.3.1-1-garuda-fallback.efi.head");

fn id(value: &str) -> LoaderEntryId {
    LoaderEntryId::new(value).unwrap()
}

fn section(image: &[u8], name: &str) -> String {
    let found = sections(image)
        .into_iter()
        .find(|section| section.name == name)
        .unwrap();

    let start = usize::try_from(found.offset).unwrap();

    let end = start + usize::try_from(found.size).unwrap();

    String::from_utf8(image[start..end].to_vec()).unwrap()
}

fn entry(file: &str, image: &[u8]) -> LoaderEntry {
    LoaderEntry::uki(
        file,
        &section(image, ".osrel"),
        Some(&section(image, ".uname")),
    )
    .unwrap()
}

/// The image's UKI before its first boot was blessed, and after.
#[test]
fn the_boot_counter_is_left_out_of_the_id() {
    for file in ["garuda-7.3.1-1-garuda+3.efi", "garuda-7.3.1-1-garuda.efi"] {
        assert_eq!(
            entry_id(file),
            Some(id("garuda-7.3.1-1-garuda.efi")),
            "{file}"
        );
    }

    assert_eq!(
        entry_id("garuda-7.3.1-1-garuda-fallback.efi"),
        Some(id("garuda-7.3.1-1-garuda-fallback.efi"))
    );
}

#[test]
fn a_file_of_another_type_is_no_entry() {
    assert_eq!(entry_id("garuda-7.3.1-1-garuda.efi.head"), None);
}

#[test]
fn the_images_ukis_have_their_os_releases_titles_and_kernel() {
    let normal = entry("garuda-7.3.1-1-garuda.efi", UKI);
    let fallback = entry("garuda-7.3.1-1-garuda-fallback.efi", FALLBACK);

    assert_eq!(normal.title, "Garuda Linux");
    assert_eq!(fallback.title, "Garuda Linux (fallback initramfs)");
    assert_eq!(
        normal.uname.as_ref().map(KernelRelease::as_str),
        Some("7.3.1-1-garuda")
    );
    assert_eq!(normal.version.as_deref(), Some("rolling"));
    assert_eq!(normal.kind, EntryKind::Uki);
    assert!(!normal.is_snapshot() && !fallback.is_snapshot());
}

/// The image's `.osrel` without `PRETTY_NAME`.
#[test]
fn without_a_pretty_name_the_name_is_the_title() {
    let os_release = section(UKI, ".osrel").replace("PRETTY_NAME", "OTHER");
    let named = LoaderEntry::uki("garuda-7.3.1-1-garuda.efi", &os_release, None).unwrap();

    assert_eq!(named.title, "Garuda Linux");
}

/// What `write_entry` of snapper-sdboot writes for a snapshot, in its own
/// field order.
#[test]
fn snapper_sdboots_entries_are_snapshots() {
    let text = "title    Garuda Linux snapshot 7\nversion  7\nsort-key snapper-sdboot\nuki      /EFI/snapper-sdboot/7/snapshot.efi\n";
    let entry = LoaderEntry::type_one("snapper-sdboot-7.conf", text).unwrap();

    assert_eq!(entry.id, id("snapper-sdboot-7.conf"));
    assert_eq!(entry.title, "Garuda Linux snapshot 7");
    assert_eq!(entry.version.as_deref(), Some("7"));
    assert_eq!(entry.kind, EntryKind::TypeOne);
    assert!(entry.is_snapshot());
}

/// The image's `.osrel` with a terminal escape in its title: the helper's
/// answer must not carry one to the terminal.
#[test]
fn control_characters_leave_the_title() {
    let os_release =
        section(UKI, ".osrel").replace("\"Garuda Linux\"", "\"Garuda\u{1b}[31m Linux\"");

    let entry = LoaderEntry::uki("garuda-7.3.1-1-garuda.efi", &os_release, None).unwrap();

    assert_eq!(entry.title, "Garuda[31m Linux");
}

fn image_menu() -> SdBootMenu {
    SdBootMenu {
        settings: LoaderSettings::default(),
        entries: vec![
            entry("garuda-7.3.1-1-garuda-fallback.efi", FALLBACK),
            entry("garuda-7.3.1-1-garuda.efi", UKI),
        ],
    }
}

/// Both of the image's UKIs carry the same `.uname`. The fallback comes
/// first in bootctl's list.
#[test]
fn a_kernels_entry_is_its_main_image_and_never_the_fallback() {
    let menu = image_menu();
    let release = KernelRelease::new("7.3.1-1-garuda").unwrap();

    assert_eq!(
        menu.kernel_entry(&release).map(|entry| entry.id.as_str()),
        Some("garuda-7.3.1-1-garuda.efi")
    );
    assert!(menu.entries[0].is_fallback());
    assert!(!menu.entries[1].is_fallback());
}

#[test]
fn a_release_without_an_image_has_no_entry() {
    let release = KernelRelease::new("6.12.9-1-lts").unwrap();

    assert!(image_menu().kernel_entry(&release).is_none());
}
