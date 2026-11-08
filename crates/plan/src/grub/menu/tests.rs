mod platform;

use super::{GrubMenu, GrubPlatform, MAX_ENTRIES, MenuEntry};
use crate::protocol::encode;

/// `grub.cfg` of a used Garuda Mokka install, every UUID masked.
const INSTALLED: &str = include_str!("../fixtures/installed/grub.cfg");
/// `grub.cfg` of the Lima image with two kernels and a test superuser.
const LIMA: &str = include_str!("../fixtures/lima/grub.cfg");
/// `grub-btrfs.cfg` of the same install, ten snapshots.
const SNAPSHOTS: &str = include_str!("../fixtures/installed/grub-btrfs.cfg");

const MASKED: &str = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
const LIMA_UUID: &str = "cd2fff21-4bb7-40f3-a158-406f545e11e6";
const PC_MEMTEST: &str = "if [ \"${grub_platform}\" == \"pc\" ]; then";

fn line(position: &str, title: &str, submenu: bool) -> (String, String, bool) {
    (position.to_owned(), title.to_owned(), submenu)
}

fn brief(menu: &GrubMenu) -> Vec<(String, String, bool)> {
    menu.entries
        .iter()
        .map(|entry| (entry.position.clone(), entry.title.clone(), entry.submenu))
        .collect()
}

fn entry<'menu>(menu: &'menu GrubMenu, title: &str) -> &'menu MenuEntry {
    menu.entries
        .iter()
        .find(|entry| entry.title == title)
        .unwrap()
}

fn found(menu: &GrubMenu, default: &str) -> Option<String> {
    menu.find(default).map(|entry| entry.title.clone())
}

fn memtests(menu: &GrubMenu) -> usize {
    menu.entries
        .iter()
        .filter(|entry| entry.title == "Memory Tester (memtest86+)")
        .count()
}

/// On UEFI the `pc` branch of memtest86+ is skipped, as GRUB does.
#[test]
fn a_garuda_menu_lists_entries_and_submenus_in_order() {
    let menu = GrubMenu::for_platform(INSTALLED, GrubPlatform::Efi);

    assert_eq!(
        brief(&menu),
        [
            line("0", "Garuda Linux", false),
            line("1", "Advanced options for Garuda Linux", true),
            line("1>0", "Garuda Linux, with Linux linux-garuda", false),
            line(
                "1>1",
                "Garuda Linux, with Linux linux-garuda (fallback initramfs)",
                false
            ),
            line(
                "1>2",
                "Garuda Linux, with Linux linux-garuda (recovery mode)",
                false
            ),
            line("2", "UEFI Firmware Settings", false),
            line("3", "Garuda Linux snapshots", true),
            line("4", "Memory Tester (memtest86+)", false),
            line("5", "Shutdown", false),
            line("6", "Restart", false),
        ]
    );
}

#[test]
fn ids_are_what_grub_default_takes() {
    let menu = GrubMenu::for_platform(INSTALLED, GrubPlatform::Efi);
    let recovery = "Garuda Linux, with Linux linux-garuda (recovery mode)";

    assert_eq!(
        entry(&menu, "Garuda Linux").id.as_str(),
        format!("gnulinux-simple-{MASKED}")
    );
    assert_eq!(
        entry(&menu, recovery).id.as_str(),
        format!("gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-recovery-{MASKED}")
    );
    assert_eq!(
        entry(&menu, recovery).parent.as_deref(),
        Some("Advanced options for Garuda Linux")
    );
    assert_eq!(entry(&menu, "Garuda Linux").parent, None);
    assert_eq!(
        entry(&menu, "UEFI Firmware Settings").id.as_str(),
        "uefi-firmware"
    );

    // Without an id of its own an entry goes by its title.
    assert_eq!(
        entry(&menu, "Memory Tester (memtest86+)").id.as_str(),
        "Memory Tester (memtest86+)"
    );
    assert_eq!(
        entry(&menu, "Garuda Linux snapshots").id.as_str(),
        "Garuda Linux snapshots"
    );
    assert_eq!(entry(&menu, "Shutdown").id.as_str(), "Shutdown");
}

/// Documents only how `grub-btrfs.cfg` parses on its own. The app never
/// reads that file, so snapshot entries are never offered as a default:
/// GRUB loads it through `configfile`, which does not carry `default` on.
/// grub-btrfs pads its titles into a table and gives every kernel of a
/// snapshot one id, so here only the first of the two is listed.
#[test]
fn snapshot_submenus_nest_and_keep_their_blanks() {
    let menu = GrubMenu::for_platform(SNAPSHOTS, GrubPlatform::Efi);
    let first = entry(
        &menu,
        "| 2026-11-07 13:48:31 | @/.snapshots/16/snapshot | post | linux-zen                                                                |",
    );

    assert_eq!(menu.entries.len(), 31);
    assert_eq!(
        menu.entries[..6]
            .iter()
            .map(|entry| entry.position.as_str())
            .collect::<Vec<_>>(),
        ["0", "1", "1>0", "1>1", "2", "2>0"]
    );
    assert!(first.submenu);
    assert_eq!(
        menu.entries[3].title,
        "  vmlinuz-linux-garuda & initramfs-linux-garuda-fallback.img"
    );
    assert_eq!(
        menu.entries[3].id.as_str(),
        format!("{}>gnulinux-snapshots-{MASKED}", first.title)
    );
    assert_eq!(
        menu.entries[3].parent.as_deref(),
        Some(first.title.as_str())
    );
    assert!(
        menu.entries
            .iter()
            .all(|entry| entry.title != "  vmlinuz-linux-garuda & initramfs-linux-garuda.img")
    );
}

/// The file may hold a password hash and the kernel command line. Only
/// titles and ids leave the helper.
#[test]
fn nothing_but_titles_and_ids_is_in_the_answer() {
    assert!(LIMA.contains("password_pbkdf2 admin grub.pbkdf2.sha512.10000.D2374BF99A3E"));

    let line = encode(&GrubMenu::for_platform(LIMA, GrubPlatform::Efi)).unwrap();

    for secret in [
        "pbkdf2",
        "D2374BF99A3E",
        "superusers",
        "admin",
        "vmlinuz",
        "root=UUID",
        "rootflags",
        "initrd",
        "insmod",
        "fwsetup",
    ] {
        assert!(!line.contains(secret), "{secret} in {line}");
    }

    assert_eq!(
        GrubMenu::parse(line.trim_end()).unwrap(),
        GrubMenu::for_platform(LIMA, GrubPlatform::Efi)
    );
}

/// No captured `grub.cfg` writes these spellings. GRUB 2.00 configs and
/// entries written by hand in `40_custom` do.
#[test]
fn old_and_odd_header_lines_are_read() {
    let config = "\
menuentry \"Double \\\"quoted\\\"\" --id old-style {\n}\n\
menuentry 'It'\\''s mine' --class x --id=inline {\n}\n\
menuentry Bare ${menuentry_id_option} 'braced' {\n}\n\
menuentry 'No id at the end' $menuentry_id_option {\n}\n\
menuentryfake 'not an entry' {\n}\n\
# menuentry 'commented' {\n\
submenu 'Open' {\n\
  menuentry 'Inside' --id in {\n\
    echo '}'\n\
  }\n\
}\n\
menuentry 'After' --id after {\n}\n";

    let menu = GrubMenu::for_platform(config, GrubPlatform::Efi);
    let ids: Vec<(&str, &str)> = menu
        .entries
        .iter()
        .map(|entry| (entry.title.as_str(), entry.id.as_str()))
        .collect();

    assert_eq!(
        ids,
        [
            ("Double \"quoted\"", "old-style"),
            ("It's mine", "inline"),
            ("Bare", "braced"),
            ("No id at the end", "No id at the end"),
            ("Open", "Open"),
            ("Inside", "Open>in"),
            ("After", "after"),
        ]
    );
    assert_eq!(menu.entries.last().unwrap().position, "5");
}

/// An id the page could not write into `/etc/default/grub` safely is not
/// offered, and neither is anything inside such a submenu.
#[test]
fn entries_with_an_unusable_id_are_left_out() {
    let config = "\
menuentry 'Fine' --id fine {\n}\n\
menuentry 'Dollar' --id 'a$b' {\n}\n\
submenu 'Bad' --id 'x`y' {\n\
  menuentry 'Lost' --id lost {\n  }\n\
}\n\
menuentry 'Last' --id last {\n}\n";

    let menu = GrubMenu::for_platform(config, GrubPlatform::Efi);

    assert_eq!(
        brief(&menu),
        [line("0", "Fine", false), line("3", "Last", false)]
    );
}

#[test]
fn titles_lose_control_characters_and_overlong_menus_are_cut() {
    let menu = GrubMenu::for_platform(
        "menuentry 'A\u{1b}[31mB\u{7}' --id a {\n}\n",
        GrubPlatform::Efi,
    );

    assert_eq!(menu.entries[0].title, "A[31mB");

    let many = (0_u32..1_100_u32)
        .map(|number| format!("menuentry 'E{number}' --id e{number} {{\n}}\n"))
        .collect::<Vec<_>>()
        .concat();

    assert_eq!(
        GrubMenu::for_platform(&many, GrubPlatform::Efi)
            .entries
            .len(),
        MAX_ENTRIES
    );
    assert_eq!(GrubMenu::for_platform("", GrubPlatform::Efi).entries, []);
    assert_eq!(
        GrubMenu::for_platform("}\n}\n{\n", GrubPlatform::Efi).entries,
        []
    );
}

#[test]
fn the_default_is_found_by_id_position_title_or_inner_id() {
    let menu = GrubMenu::for_platform(INSTALLED, GrubPlatform::Efi);
    let found = |default: &str| menu.find(default).map(|entry| entry.title.as_str());

    assert_eq!(
        found(&format!("gnulinux-simple-{MASKED}")),
        Some("Garuda Linux")
    );
    assert_eq!(found("0"), Some("Garuda Linux"));
    assert_eq!(
        found("1>2"),
        Some("Garuda Linux, with Linux linux-garuda (recovery mode)")
    );
    assert_eq!(found("Garuda Linux"), Some("Garuda Linux"));
    assert_eq!(
        found(&format!("gnulinux-linux-garuda-fallback-{MASKED}")),
        Some("Garuda Linux, with Linux linux-garuda (fallback initramfs)")
    );
    assert_eq!(found("saved"), None);
    assert_eq!(found("9"), None);
}

#[test]
fn an_answer_with_a_bad_id_is_refused_whole() {
    GrubMenu::parse(
        r#"{"entries":[{"id":"a$b","title":"x","position":"0","submenu":false,"parent":null}]}"#,
    )
    .unwrap_err();
    GrubMenu::parse("[]").unwrap_err();
    assert_eq!(
        GrubMenu::parse(r#"{"entries":[]}"#).unwrap(),
        GrubMenu::default()
    );
}

#[test]
fn a_kernel_finds_the_entry_that_boots_it_and_not_its_fallback() {
    let menu = GrubMenu::for_platform(LIMA, GrubPlatform::Efi);
    let kernel = |name: &str| crate::PackageName::new(name).unwrap();
    let garuda = menu.kernel_entry(&kernel("linux-garuda")).unwrap();

    assert_eq!(garuda.title, "Garuda Linux, with Linux linux-garuda");
    assert_eq!(
        garuda.id.as_str(),
        format!("gnulinux-advanced-{LIMA_UUID}>gnulinux-linux-garuda-advanced-{LIMA_UUID}")
    );
    assert_eq!(
        menu.kernel_entry(&kernel("linux-lts"))
            .map(|entry| entry.title.as_str()),
        Some("Garuda Linux, with Linux linux-lts")
    );
    assert_eq!(
        menu.kernel_entry(&kernel("linux")),
        None,
        "linux is not installed here, and linux-lts is another kernel"
    );
    assert_eq!(menu.kernel_entry(&kernel("linux-garuda-headers")), None);
    assert_eq!(GrubMenu::default().kernel_entry(&kernel("linux-lts")), None);
}

/// The memtest86+ BIOS and EFI scripts both write an entry without an id
/// and with one title. GRUB boots the first for that title. The test
/// turns the `pc` check into an `efi` one, so both are in the menu.
#[test]
fn an_id_twice_is_listed_once_as_grub_boots_the_first() {
    let both = INSTALLED.replacen(
        PC_MEMTEST,
        "if [ \"${grub_platform}\" == \"efi\" ]; then",
        1,
    );

    assert_eq!(
        both.matches("menuentry \"Memory Tester (memtest86+)\"")
            .count(),
        2
    );
    assert_ne!(both, INSTALLED);

    let menu = GrubMenu::for_platform(&both, GrubPlatform::Efi);

    assert_eq!(found(&menu, "6").as_deref(), Some("Shutdown"));
    assert_eq!(memtests(&menu), 1);
}
