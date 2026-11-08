use toolbox_plan::{DefaultEntry, GrubKey, GrubMenu, GrubSetting, GrubTimeout, MenuEntryId};

use super::{
    EntryRow, Page, Probed, Readiness, entries, is_queued, parameters_line, plymouth_current,
    plymouth_installed, plymouth_shown, plymouth_theme_queued, plymouth_themes, state,
    virtual_machine,
};
use crate::features::boot::tests::{system, with_plymouth};
use crate::features::boot::{
    Flag, VirtualMachine, plymouth_queued, set, set_flag, set_plymouth_theme, set_remember,
};
use crate::testing::plymouth_theme;
use crate::{ChangeSet, Snapshot, Toggle};

const DEFAULTS: &str = "\
GRUB_DEFAULT=gnulinux-simple-1
GRUB_TIMEOUT=5
GRUB_CMDLINE_LINUX_DEFAULT=\"quiet splash loglevel=3\"
GRUB_THEME=\"/usr/share/grub/themes/garuda/theme.txt\"
";

const GRUB_CFG: &str = "\
menuentry 'Garuda Linux' $menuentry_id_option 'gnulinux-simple-1' {
}
submenu 'Advanced options' $menuentry_id_option 'gnulinux-advanced-1' {
	menuentry 'Garuda Linux, with Linux linux-lts' $menuentry_id_option 'gnulinux-lts-1' {
	}
}
";

fn menu() -> GrubMenu {
    GrubMenu::from_config(GRUB_CFG)
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
    assert_eq!(
        entries(Some(&menu)),
        [
            EntryRow {
                entry: "gnulinux-simple-1".to_owned(),
                title: "Garuda Linux".to_owned(),
                parent: String::new(),
            },
            EntryRow {
                entry: "gnulinux-advanced-1>gnulinux-lts-1".to_owned(),
                title: "Garuda Linux, with Linux linux-lts".to_owned(),
                parent: "Advanced options".to_owned(),
            },
        ]
    );
    let page = Page::read(&snapshot, &changes);
    assert_eq!(page.entry_value(Some(&menu)), "gnulinux-simple-1");
    assert!(page.entry_listed(Some(&menu)));
    assert!(!is_queued(&changes, GrubKey::Default));
    assert_eq!(page.timeout(), 5_i32);
    assert!(page.timeout_known());
    assert!(!is_queued(&changes, GrubKey::Timeout));
    assert!(!page.remember());
    assert!(!is_queued(&changes, GrubKey::SaveDefault));
    assert!(page.submenus());
    assert!(!is_queued(&changes, GrubKey::Submenus));
    assert!(!page.os_prober());
    assert!(!is_queued(&changes, GrubKey::OsProber));
    assert!(page.parameters_editable());
    assert_eq!(parameters_line(&snapshot), "quiet splash loglevel=3");
    assert_eq!(page.parameter_items(), ["loglevel=3"]);
    assert!(page.flag(Flag::Quiet));
    assert!(page.flag(Flag::Splash));
    assert_eq!(page.flags_queued(), Vec::<&str>::new());
    assert!(!page.flag(Flag::MitigationsOff));
    assert!(!page.flag(Flag::SysRq));
    assert!(!is_queued(&changes, GrubKey::KernelParameters));
    assert_eq!(page.theme(), "/usr/share/grub/themes/garuda/theme.txt");
    assert!(!is_queued(&changes, GrubKey::Theme));
    assert_eq!(page.background(), "");
    assert!(!is_queued(&changes, GrubKey::Background));
    assert!(!page.background_from_theme());
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
    let mut state = with_plymouth(system(DEFAULTS, &[]));
    state.insert(VirtualMachine(true));

    let mut changes = ChangeSet::default();

    set(
        GrubSetting::Timeout(GrubTimeout::new(12).unwrap()),
        &mut changes,
        &state,
    )
    .unwrap();
    set_flag(Flag::MitigationsOff, Toggle::On, &mut changes, &state).unwrap();
    set_flag(Flag::Splash, Toggle::Off, &mut changes, &state).unwrap();
    set(
        GrubSetting::Default(DefaultEntry::Entry(
            MenuEntryId::new("gnulinux-advanced-1>gnulinux-lts-1").unwrap(),
        )),
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
        "gnulinux-advanced-1>gnulinux-lts-1"
    );
    assert!(is_queued(&changes, GrubKey::Default));
    assert_eq!(
        Page::read(&state, &changes).parameter_items(),
        ["loglevel=3"]
    );
    assert!(is_queued(&changes, GrubKey::KernelParameters));
    assert!(!Page::read(&state, &changes).flag(Flag::Splash));
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
        "GRUB_CMDLINE_LINUX_DEFAULT=\"sysrq mitigations=auto,nosmt loglevel=3\"\n",
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
    let state = system("GRUB_CMDLINE_LINUX_DEFAULT=quiet\n", &[]);
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
    let value = |defaults: &str, menu: Option<&GrubMenu>| -> String {
        Page::read(&system(defaults, &[]), &ChangeSet::default()).entry_value(menu)
    };

    let listed = |defaults: &str, menu: Option<&GrubMenu>| -> bool {
        Page::read(&system(defaults, &[]), &ChangeSet::default()).entry_listed(menu)
    };

    assert_eq!(
        value("GRUB_DEFAULT=0\n", Some(&menu())),
        "gnulinux-simple-1"
    );
    assert!(listed("GRUB_DEFAULT=0\n", Some(&menu())));
    assert_eq!(
        value("GRUB_DEFAULT='1>0'\n", Some(&menu())),
        "gnulinux-advanced-1>gnulinux-lts-1"
    );
    assert_eq!(
        value("GRUB_DEFAULT='Garuda Linux'\n", Some(&menu())),
        "gnulinux-simple-1"
    );
    assert_eq!(value("GRUB_DEFAULT=1\n", Some(&menu())), "1");
    assert!(!listed("GRUB_DEFAULT=1\n", Some(&menu())));
    assert_eq!(value("GRUB_DEFAULT=gone\n", Some(&menu())), "gone");
    assert_eq!(value("GRUB_DEFAULT=0\n", None), "0");
}

#[test]
fn a_command_line_the_page_leaves_alone_is_shown_as_written() {
    let snapshot = system(
        "GRUB_CMDLINE_LINUX_DEFAULT='quiet acpi_osi=\"Windows 2015\"'\nGRUB_TIMEOUT=soon\n",
        &[],
    );

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
fn a_stock_garuda_file_shows_the_picture_of_its_theme() {
    let defaults = format!(
        "{DEFAULTS}export GRUB_MENU_PICTURE=\"/usr/share/grub/themes/garuda/background.png\"\n"
    );

    let snapshot = system(&defaults, &[]);
    let changes = ChangeSet::default();

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
