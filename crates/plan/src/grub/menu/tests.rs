use super::{GrubMenu, MAX_ENTRIES, MenuEntry};
use crate::protocol::encode;

const GARUDA: &str = include_str!("../fixtures/garuda-grub.cfg");
const SNAPSHOTS: &str = include_str!("../fixtures/snapshots-grub-btrfs.cfg");

const UUID: &str = "3f1c9a52-7d0e-4b6a-9c1f-2e8d5a7b4c10";

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

#[test]
fn a_garuda_menu_lists_entries_and_submenus_in_order() {
    let menu = GrubMenu::from_config(GARUDA);

    assert_eq!(
        brief(&menu),
        [
            line("0", "Garuda Linux", false),
            line("1", "Advanced options for Garuda Linux", true),
            line("1>0", "Garuda Linux, with Linux linux-zen", false),
            line(
                "1>1",
                "Garuda Linux, with Linux linux-zen (fallback initramfs)",
                false
            ),
            line("1>2", "Garuda Linux, with Linux linux-lts", false),
            line("2", "Windows Boot Manager (on /dev/nvme0n1p1)", false),
            line("3", "UEFI Firmware Settings", false),
            line("4", "Garuda Linux snapshots", true),
            line("5", "Memory Tester (memtest86+)", false),
        ]
    );
}

#[test]
fn ids_are_what_grub_default_takes() {
    let menu = GrubMenu::from_config(GARUDA);

    assert_eq!(
        entry(&menu, "Garuda Linux").id.as_str(),
        format!("gnulinux-simple-{UUID}")
    );
    assert_eq!(
        entry(&menu, "Garuda Linux, with Linux linux-lts")
            .id
            .as_str(),
        format!("gnulinux-advanced-{UUID}>gnulinux-linux-lts-advanced-{UUID}")
    );
    assert_eq!(
        entry(&menu, "Garuda Linux, with Linux linux-lts")
            .parent
            .as_deref(),
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
}

#[test]
fn snapshot_submenus_nest_and_keep_their_blanks() {
    let menu = GrubMenu::from_config(SNAPSHOTS);
    let first = "  2026-10-09 18:20:11 | @/.snapshots/412/snapshot | pre  | pacman -Syu";

    assert_eq!(menu.entries.len(), 6);
    assert_eq!(
        menu.entries
            .iter()
            .map(|entry| entry.position.as_str())
            .collect::<Vec<_>>(),
        ["0", "0>0", "0>0>0", "0>0>1", "0>1", "0>1>0"]
    );
    assert_eq!(
        menu.entries[2].id.as_str(),
        format!("Garuda Linux snapshots>{first}>gnulinux-snapshots-3f1c9a52-412-zen")
    );
    assert_eq!(menu.entries[2].parent.as_deref(), Some(first));
    assert_eq!(
        menu.entries[5].id.as_str(),
        "Garuda Linux snapshots>  2026-10-08 09:02:44 | @/.snapshots/411/snapshot | post | linux-zen>gnulinux-snapshots-3f1c9a52-411-zen"
    );
}

/// The file may hold a password hash and the kernel command line. Only
/// titles and ids leave the helper.
#[test]
fn nothing_but_titles_and_ids_is_in_the_answer() {
    let line = encode(&GrubMenu::from_config(GARUDA)).unwrap();

    for secret in [
        "pbkdf2",
        "5A9F0C3E",
        "superusers",
        "admin",
        "vmlinuz",
        "root=UUID",
        "rootflags",
        "initrd",
        "insmod",
        "chainloader",
    ] {
        assert!(!line.contains(secret), "{secret} in {line}");
    }

    assert_eq!(
        GrubMenu::parse(line.trim_end()).unwrap(),
        GrubMenu::from_config(GARUDA)
    );
}

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

    let menu = GrubMenu::from_config(config);
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

    let menu = GrubMenu::from_config(config);

    assert_eq!(
        brief(&menu),
        [line("0", "Fine", false), line("3", "Last", false)]
    );
}

#[test]
fn titles_lose_control_characters_and_overlong_menus_are_cut() {
    let menu = GrubMenu::from_config("menuentry 'A\u{1b}[31mB\u{7}' --id a {\n}\n");

    assert_eq!(menu.entries[0].title, "A[31mB");

    let many = (0_u32..1_100_u32)
        .map(|number| format!("menuentry 'E{number}' --id e{number} {{\n}}\n"))
        .collect::<Vec<_>>()
        .concat();

    assert_eq!(GrubMenu::from_config(&many).entries.len(), MAX_ENTRIES);
    assert_eq!(GrubMenu::from_config("").entries, []);
    assert_eq!(GrubMenu::from_config("}\n}\n{\n").entries, []);
}

#[test]
fn the_default_is_found_by_id_position_title_or_inner_id() {
    let menu = GrubMenu::from_config(GARUDA);
    let found = |default: &str| menu.find(default).map(|entry| entry.title.as_str());

    assert_eq!(
        found(&format!("gnulinux-simple-{UUID}")),
        Some("Garuda Linux")
    );
    assert_eq!(found("0"), Some("Garuda Linux"));
    assert_eq!(found("1>2"), Some("Garuda Linux, with Linux linux-lts"));
    assert_eq!(found("Garuda Linux"), Some("Garuda Linux"));
    assert_eq!(
        found(&format!("gnulinux-linux-zen-fallback-{UUID}")),
        Some("Garuda Linux, with Linux linux-zen (fallback initramfs)")
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
    let menu = GrubMenu::from_config(GARUDA);
    let kernel = |name: &str| crate::PackageName::new(name).unwrap();
    let zen = menu.kernel_entry(&kernel("linux-zen")).unwrap();

    assert_eq!(zen.title, "Garuda Linux, with Linux linux-zen");
    assert_eq!(
        zen.id.as_str(),
        format!("gnulinux-advanced-{UUID}>gnulinux-linux-zen-advanced-{UUID}")
    );
    assert_eq!(
        menu.kernel_entry(&kernel("linux-lts"))
            .map(|entry| entry.title.as_str()),
        Some("Garuda Linux, with Linux linux-lts")
    );
    assert_eq!(
        menu.kernel_entry(&kernel("linux")),
        None,
        "linux is not installed here, and linux-zen is another kernel"
    );
    assert_eq!(menu.kernel_entry(&kernel("linux-zen-headers")), None);
    assert_eq!(GrubMenu::default().kernel_entry(&kernel("linux-zen")), None);
}

/// Two entries without an id and with one title, as the memtest86+ EFI
/// and BIOS scripts write them. GRUB boots the first for that title.
#[test]
fn an_id_twice_is_listed_once_as_grub_boots_the_first() {
    let config = "\
menuentry 'Memory Tester (memtest86+)' {\n}\n\
menuentry 'Memory Tester (memtest86+)' {\n}\n";

    let menu = GrubMenu::from_config(config);

    assert_eq!(
        brief(&menu),
        [line("0", "Memory Tester (memtest86+)", false)]
    );
}
