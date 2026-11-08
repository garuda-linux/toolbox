use toolbox_plan::{
    DefaultEntry, GrubKey, GrubMenu, GrubPlatform, GrubSetting, GrubTimeout, MenuEntryId,
};

use super::{
    Page, Probed, Readiness, entries, is_queued, parameters_line, plymouth_current,
    plymouth_installed, plymouth_shown, plymouth_theme_queued, plymouth_themes, state,
    virtual_machine,
};
use crate::features::boot::tests::{system, with_plymouth};
use crate::features::boot::{
    Flag, VirtualMachine, plymouth_queued, set, set_flag, set_plymouth_theme, set_remember,
};
use crate::testing::plymouth_theme;
use crate::{ChangeSet, Snapshot, Toggle};

/// `/etc/default/grub` and `grub.cfg` of a Garuda Mokka install, UUIDs
/// of `grub.cfg` replaced by `x`.
const DEFAULTS: &str = include_str!("../../../../../plan/src/grub/fixtures/installed/default-grub");
const GRUB_CFG: &str = include_str!("../../../../../plan/src/grub/fixtures/installed/grub.cfg");
/// `/etc/default/grub` of the Lima VM, with Garuda's own GRUB theme.
const LIMA_DEFAULTS: &str = include_str!("../../../../../plan/src/grub/fixtures/lima/default-grub");

const MASKED: &str = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";
const COMMAND_LINE: &str = "GRUB_CMDLINE_LINUX_DEFAULT='quiet loglevel=3'";

fn menu() -> GrubMenu {
    GrubMenu::for_platform(GRUB_CFG, GrubPlatform::Efi)
}

fn simple() -> String {
    format!("gnulinux-simple-{MASKED}")
}

fn fallback() -> String {
    format!("gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-fallback-{MASKED}")
}

/// [`DEFAULTS`] with `from` edited to `to`.
fn edited(from: &str, to: &str) -> String {
    let edited = DEFAULTS.replace(from, to);

    assert_ne!(edited, DEFAULTS, "{from} is in the file");
    edited
}

#[test]
fn before_the_first_probe_the_page_loads_and_without_grub_it_says_so() {
    let none = (Snapshot::default(), ChangeSet::default());
    let read = Probed {
        grub_failed: false,
        read: true,
    };

    assert_eq!(state(&none.0, Probed::default()), Readiness::Loading);
    assert_eq!(state(&none.0, read), Readiness::Missing);
}

#[test]
fn a_grub_file_that_could_not_be_read_is_no_missing_grub() {
    let failed = Probed {
        grub_failed: true,
        read: true,
    };

    let state_of = state(&Snapshot::default(), failed);
    assert_eq!(state_of, Readiness::Failed);
}

#[test]
fn a_grub_system_shows_its_file_and_menu() {
    let snapshot = system(DEFAULTS, &[]);
    let changes = ChangeSet::default();
    let menu = menu();

    assert_eq!(state(&snapshot, Probed::default()), Readiness::Ready);
    assert!(!virtual_machine(&snapshot));

    let page = Page::read(&snapshot, &changes);
    assert_eq!(page.entry_value(Some(&menu)), simple());
    assert!(page.entry_listed(Some(&menu)));
    assert!(!is_queued(&changes, GrubKey::Default));
    assert_eq!(page.timeout(), 5_i32);
    assert!(page.timeout_known());
    assert!(!is_queued(&changes, GrubKey::Timeout));
    assert!(!page.remember());
    assert!(!is_queued(&changes, GrubKey::SaveDefault));
    assert!(page.submenus());
    assert!(!is_queued(&changes, GrubKey::Submenus));
    assert!(page.os_prober());
    assert!(!is_queued(&changes, GrubKey::OsProber));
    assert!(page.parameters_editable());
    assert_eq!(parameters_line(&snapshot), "quiet loglevel=3");
    assert_eq!(page.parameter_items(), ["loglevel=3"]);
    assert!(page.flag(Flag::Quiet));
    assert!(!page.flag(Flag::Splash));
    assert_eq!(page.flags_queued(), Vec::<&str>::new());
    assert!(!page.flag(Flag::MitigationsOff));
    assert!(!page.flag(Flag::SysRq));
    assert!(!is_queued(&changes, GrubKey::KernelParameters));
    assert_eq!(
        page.theme(),
        "/usr/share/grub/themes/catppuccin-mocha/theme.txt"
    );
    assert!(!is_queued(&changes, GrubKey::Theme));
    assert_eq!(page.background(), "");
    assert!(!is_queued(&changes, GrubKey::Background));
    assert!(!page.background_from_theme());
}

#[test]
fn a_grub_menu_lists_every_entry_under_its_submenu() {
    let menu = menu();
    let rows: Vec<(String, String, String)> = entries(Some(&menu))
        .into_iter()
        .map(|row| (row.entry, row.title, row.parent))
        .collect();

    let advanced = "Advanced options for Garuda Linux";
    let row = |entry: &str, title: &str, parent: &str| {
        (entry.to_owned(), title.to_owned(), parent.to_owned())
    };

    assert_eq!(
        rows,
        [
            row(&simple(), "Garuda Linux", ""),
            row(
                &format!("gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-advanced-{MASKED}"),
                "Garuda Linux, with Linux linux-garuda",
                advanced
            ),
            row(
                &fallback(),
                "Garuda Linux, with Linux linux-garuda (fallback initramfs)",
                advanced
            ),
            row(
                &format!("gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-recovery-{MASKED}"),
                "Garuda Linux, with Linux linux-garuda (recovery mode)",
                advanced
            ),
            row("uefi-firmware", "UEFI Firmware Settings", ""),
            row(
                "Memory Tester (memtest86+)",
                "Memory Tester (memtest86+)",
                ""
            ),
            row("Shutdown", "Shutdown", ""),
            row("Restart", "Restart", ""),
        ]
    );
}

#[test]
fn a_grub_system_without_plymouth_has_no_themes() {
    let snapshot = system(DEFAULTS, &[]);
    let changes = ChangeSet::default();

    assert!(!plymouth_installed(&snapshot));
    assert!(!plymouth_queued(&changes));
    assert_eq!(plymouth_themes(&snapshot), Vec::<String>::new());
    assert_eq!(plymouth_current(&snapshot), "");
    assert_eq!(plymouth_theme_queued(&changes), "");
}

#[test]
fn queued_changes_show_on_top() {
    let mut state = with_plymouth(system(DEFAULTS, &["plymouth"]));
    state.insert(VirtualMachine(true));

    let mut changes = ChangeSet::default();

    set(
        GrubSetting::Timeout(GrubTimeout::new(12).unwrap()),
        &mut changes,
        &state,
    )
    .unwrap();
    set_flag(Flag::MitigationsOff, Toggle::On, &mut changes, &state).unwrap();
    set_flag(Flag::Splash, Toggle::On, &mut changes, &state).unwrap();
    set(
        GrubSetting::Default(DefaultEntry::Entry(MenuEntryId::new(fallback()).unwrap())),
        &mut changes,
        &state,
    )
    .unwrap();
    set_plymouth_theme(plymouth_theme("spinner"), &mut changes, &state).unwrap();

    let menu = menu();

    assert!(virtual_machine(&state));
    assert_eq!(Page::read(&state, &changes).timeout(), 12_i32);
    assert!(is_queued(&changes, GrubKey::Timeout));
    assert_eq!(
        Page::read(&state, &changes).entry_value(Some(&menu)),
        fallback()
    );
    assert!(is_queued(&changes, GrubKey::Default));
    assert_eq!(
        Page::read(&state, &changes).parameter_items(),
        ["loglevel=3"]
    );
    assert!(is_queued(&changes, GrubKey::KernelParameters));
    assert!(Page::read(&state, &changes).flag(Flag::Splash));
    assert!(Page::read(&state, &changes).flag(Flag::MitigationsOff));
    assert_eq!(
        Page::read(&state, &changes).flags_queued(),
        ["splash", "mitigations-off"]
    );
    assert!(plymouth_installed(&state));
    assert!(!plymouth_queued(&changes));
    assert_eq!(plymouth_themes(&state), ["bgrt", "spinner"]);
    assert_eq!(plymouth_current(&state), "bgrt");
    assert_eq!(plymouth_theme_queued(&changes), "spinner");
}

#[test]
fn the_free_list_hides_both_sysrq_spellings_but_keeps_other_mitigations() {
    let snapshot = system(
        &edited(
            COMMAND_LINE,
            "GRUB_CMDLINE_LINUX_DEFAULT=\"sysrq mitigations=auto,nosmt loglevel=3\"",
        ),
        &[],
    );

    let changes = ChangeSet::default();

    assert_eq!(
        Page::read(&snapshot, &changes).parameter_items(),
        ["mitigations=auto,nosmt", "loglevel=3"]
    );
    assert!(!Page::read(&snapshot, &changes).flag(Flag::Quiet));
    assert!(!Page::read(&snapshot, &changes).flag(Flag::Splash));
    assert!(!Page::read(&snapshot, &changes).flag(Flag::MitigationsOff));
    assert!(Page::read(&snapshot, &changes).flag(Flag::SysRq));
}

#[test]
fn remembering_empties_the_default_and_the_splash_queues_plymouth() {
    let state = system(DEFAULTS, &[]);
    let mut changes = ChangeSet::default();

    set_remember(true, MenuEntryId::new("0").unwrap(), &mut changes, &state).unwrap();
    set_flag(Flag::Splash, Toggle::On, &mut changes, &state).unwrap();

    let menu = menu();

    assert!(Page::read(&state, &changes).remember());
    assert!(is_queued(&changes, GrubKey::SaveDefault));
    assert_eq!(Page::read(&state, &changes).entry_value(Some(&menu)), "");
    assert!(!Page::read(&state, &changes).entry_listed(Some(&menu)));
    assert!(plymouth_queued(&changes));
}

#[test]
fn the_default_is_matched_by_number_or_title_and_else_shown_as_written() {
    let page = |default: Option<&str>| {
        let defaults = default.map_or_else(
            || DEFAULTS.to_owned(),
            |value| edited("GRUB_DEFAULT=0\n", &format!("GRUB_DEFAULT={value}\n")),
        );

        Page::read(&system(&defaults, &[]), &ChangeSet::default())
    };

    let menu = menu();
    let value = |default: Option<&str>| page(default).entry_value(Some(&menu));

    assert_eq!(value(None), simple());
    assert!(page(None).entry_listed(Some(&menu)));
    assert_eq!(value(Some("'1>1'")), fallback());
    assert_eq!(value(Some("'Garuda Linux'")), simple());
    assert_eq!(value(Some("1")), "1");
    assert!(!page(Some("1")).entry_listed(Some(&menu)));
    assert_eq!(value(Some("gone")), "gone");
    assert_eq!(page(None).entry_value(None), "0");
}

#[test]
fn a_command_line_the_page_leaves_alone_is_shown_as_written() {
    let defaults = edited(
        COMMAND_LINE,
        "GRUB_CMDLINE_LINUX_DEFAULT='quiet acpi_osi=\"Windows 2015\"'",
    )
    .replace("GRUB_TIMEOUT=5\n", "GRUB_TIMEOUT=soon\n");

    let snapshot = system(&defaults, &[]);

    let changes = ChangeSet::default();

    assert!(!Page::read(&snapshot, &changes).parameters_editable());
    assert_eq!(
        parameters_line(&snapshot),
        "quiet acpi_osi=\"Windows 2015\""
    );
    assert_eq!(
        Page::read(&snapshot, &changes).parameter_items(),
        Vec::<String>::new()
    );
    assert!(!Page::read(&snapshot, &changes).flag(Flag::Quiet));
    assert_eq!(Page::read(&snapshot, &changes).timeout(), 5_i32);
    assert!(!Page::read(&snapshot, &changes).timeout_known());
}

#[test]
fn a_menu_picture_line_shows_the_picture_of_its_theme() {
    // Edit: no captured file sets `GRUB_MENU_PICTURE`, so the Lima file gets
    // the line garuda-boot-options writes.
    let defaults = format!(
        "{LIMA_DEFAULTS}export GRUB_MENU_PICTURE=\"/usr/share/grub/themes/garuda/background.png\"\n"
    );

    let snapshot = system(&defaults, &[]);
    let changes = ChangeSet::default();

    assert_eq!(
        Page::read(&snapshot, &changes).theme(),
        "/usr/share/grub/themes/garuda/theme.txt"
    );
    assert_eq!(
        Page::read(&snapshot, &changes).background(),
        "/usr/share/grub/themes/garuda/background.png"
    );
    assert!(Page::read(&snapshot, &changes).background_from_theme());
}

#[test]
fn the_queued_plymouth_theme_shows_instead_of_the_current_one() {
    let state = with_plymouth(system(DEFAULTS, &[]));

    assert_eq!(
        plymouth_shown(&system(DEFAULTS, &[]), &ChangeSet::default()),
        ""
    );

    let mut changes = ChangeSet::default();

    assert_eq!(plymouth_shown(&state, &changes), "bgrt");

    set_plymouth_theme(plymouth_theme("spinner"), &mut changes, &state).unwrap();

    assert_eq!(plymouth_shown(&state, &changes), "spinner");
}
