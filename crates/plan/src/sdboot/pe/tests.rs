use super::{Section, sections};

/// The first bytes of the Garuda Lima image's UKI, up to the end of
/// `.uname`.
const UKI: &[u8] = crate::testing::sdboot::GARUDA_7_3_1_1_GARUDA_EFI_HEAD;

/// As `objdump -h` lists them for that UKI.
#[test]
fn the_images_uki_has_its_small_sections_at_objdumps_offsets() {
    let found = sections(UKI);
    let named = |name: &str| found.iter().find(|section| section.name == name).cloned();

    assert_eq!(
        named(".osrel"),
        Some(Section {
            name: ".osrel".to_owned(),
            offset: 0x20c00,
            size: 0x1b2,
        })
    );
    assert_eq!(
        named(".uname"),
        Some(Section {
            name: ".uname".to_owned(),
            offset: 0x21000,
            size: 0xe,
        })
    );
    assert_eq!(found.len(), 11);
}

/// The image's UKI with its `MZ` or its `PE` signature broken.
#[test]
fn a_broken_signature_leaves_no_sections() {
    let mut not_mz = UKI.to_vec();
    let mut not_pe = UKI.to_vec();
    let pe = usize::try_from(u32::from_le_bytes(UKI[0x3c..0x40].try_into().unwrap())).unwrap();

    not_mz[0] = b'X';
    not_pe[pe] = b'X';

    assert_eq!(sections(&not_mz), Vec::<Section>::new());
    assert_eq!(sections(&not_pe), Vec::<Section>::new());
    assert_eq!(sections(&UKI[..2]), Vec::<Section>::new());
}

/// The image's UKI with its PE offset pointing past the end.
#[test]
fn a_header_that_points_outside_has_no_sections() {
    let mut bytes = UKI.to_vec();

    bytes[0x3c..0x40].copy_from_slice(&u32::MAX.to_le_bytes());

    assert_eq!(sections(&bytes), Vec::<Section>::new());
}

/// The image's UKI cut inside its section table keeps the rows before.
#[test]
fn a_cut_header_keeps_the_sections_it_holds() {
    let whole = sections(UKI);
    let pe = usize::try_from(u32::from_le_bytes(UKI[0x3c..0x40].try_into().unwrap())).unwrap();
    let optional = usize::from(u16::from_le_bytes(
        UKI[pe + 20..pe + 22].try_into().unwrap(),
    ));

    let table = pe + 24 + optional;

    assert_eq!(sections(&UKI[..table + 3 * 40]), whole[..3]);
}
