mod background;
mod flags;
mod recheck;

use toolbox_plan::{
    DefaultEntry, GrubKey, GrubPicture, GrubSetting, GrubState, GrubThemeFile, GrubTimeout,
    KernelParameter, MenuEntryId,
};

use super::{
    Flag, Refusal, add_parameter, parameters, plymouth_queued, remove_parameter, set, set_flag,
    set_plymouth_theme, set_remember, shown,
};
use crate::domains::{
    CurrentGrub, GrubDefaults, InstalledPackages, Packages, PlymouthState, PlymouthThemes,
};
use crate::testing::{package, packages, plymouth_theme};
use crate::{ChangeSet, Single, Snapshot, Toggle};

pub(super) const ARCH: &str =
    "GRUB_DEFAULT=0\nGRUB_TIMEOUT=5\nGRUB_CMDLINE_LINUX_DEFAULT=\"loglevel=3 quiet\"\n";

pub(super) fn system(defaults: &str, installed: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(CurrentGrub(GrubState::read(defaults)));
    snapshot.insert(InstalledPackages(packages(installed)));
    snapshot
}

pub(super) fn with_plymouth(mut snapshot: Snapshot) -> Snapshot {
    snapshot.insert(PlymouthState {
        themes: vec![plymouth_theme("bgrt"), plymouth_theme("spinner")],
        current: Some(plymouth_theme("bgrt")),
    });
    snapshot
}

fn entry(id: &str) -> MenuEntryId {
    MenuEntryId::new(id).unwrap()
}

fn parameter(text: &str) -> KernelParameter {
    KernelParameter::new(text).unwrap()
}

fn queued_line(changes: &ChangeSet) -> Option<String> {
    match GrubDefaults::queued(changes, GrubKey::KernelParameters) {
        Some(GrubSetting::KernelParameters(list)) => Some(list.to_string()),
        _ => None,
    }
}

#[test]
fn without_the_grub_file_nothing_is_shown_and_nothing_queues() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();

    assert_eq!(shown(&empty, &changes), None);
    assert_eq!(
        set(
            GrubSetting::Timeout(GrubTimeout::new(3).unwrap()),
            &mut changes,
            &empty
        ),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        set(
            GrubSetting::Default(DefaultEntry::Entry(entry("0"))),
            &mut changes,
            &empty
        ),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        set_flag(Flag::Splash, Toggle::On, &mut changes, &empty),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        add_parameter(parameter("quiet"), &mut changes, &empty),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        set_remember(true, entry("0"), &mut changes, &empty),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        set(GrubSetting::Background(None), &mut changes, &empty),
        Err(Refusal::Unavailable)
    );
    assert_eq!(
        set(GrubSetting::Theme(None), &mut changes, &empty),
        Err(Refusal::Unavailable)
    );
    assert!(changes.is_empty());
}

#[test]
fn the_page_shows_the_file_with_the_queue_on_top() {
    let state = system(ARCH, &[]);
    let mut changes = ChangeSet::default();
    let before = shown(&state, &changes).unwrap();

    assert_eq!(before.default, "0");
    assert_eq!(before.timeout, Some(5));
    assert!(!before.remember && before.submenus && !before.os_prober);

    set(
        GrubSetting::Timeout(GrubTimeout::new(9).unwrap()),
        &mut changes,
        &state,
    )
    .unwrap();
    set(
        GrubSetting::Default(DefaultEntry::Entry(entry("a>b"))),
        &mut changes,
        &state,
    )
    .unwrap();
    set(
        GrubSetting::Background(Some(GrubPicture::new("/usr/share/a.png").unwrap())),
        &mut changes,
        &state,
    )
    .unwrap();
    set(
        GrubSetting::Theme(Some(
            GrubThemeFile::new("/usr/share/grub/themes/x/theme.txt").unwrap(),
        )),
        &mut changes,
        &state,
    )
    .unwrap();

    let after = shown(&state, &changes).unwrap();

    assert_eq!(after.default, "a>b");
    assert_eq!(after.timeout, Some(9));
    assert_eq!(after.background, "/usr/share/a.png");
    assert_eq!(after.theme, "/usr/share/grub/themes/x/theme.txt");
    assert_eq!(changes.len(), 4);
}

#[test]
fn parameters_are_added_once_and_removed_from_the_queued_list() {
    let state = system(ARCH, &[]);
    let mut changes = ChangeSet::default();

    add_parameter(parameter("nowatchdog"), &mut changes, &state).unwrap();
    add_parameter(parameter("nowatchdog"), &mut changes, &state).unwrap();
    add_parameter(parameter("ibt=off"), &mut changes, &state).unwrap();
    assert_eq!(
        queued_line(&changes).as_deref(),
        Some("loglevel=3 quiet nowatchdog ibt=off")
    );

    remove_parameter("loglevel=3", &mut changes, &state).unwrap();
    assert_eq!(
        parameters(&state, &changes).unwrap().to_string(),
        "quiet nowatchdog ibt=off"
    );

    remove_parameter("nowatchdog", &mut changes, &state).unwrap();
    remove_parameter("ibt=off", &mut changes, &state).unwrap();
    add_parameter(parameter("loglevel=3"), &mut changes, &state).unwrap();

    // The same words in another order are another command line.
    assert_eq!(queued_line(&changes).as_deref(), Some("quiet loglevel=3"));
    assert_eq!(changes.len(), 1);
}

#[test]
fn taking_an_added_parameter_back_empties_the_queue() {
    let state = system(ARCH, &[]);
    let mut changes = ChangeSet::default();

    add_parameter(parameter("nowatchdog"), &mut changes, &state).unwrap();
    remove_parameter("nowatchdog", &mut changes, &state).unwrap();

    assert!(changes.is_empty());
}

#[test]
fn the_switches_write_their_parameters() {
    let state = system(ARCH, &["plymouth"]);
    let mut changes = ChangeSet::default();

    set_flag(Flag::MitigationsOff, Toggle::On, &mut changes, &state).unwrap();
    set_flag(Flag::SysRq, Toggle::On, &mut changes, &state).unwrap();
    set_flag(Flag::Splash, Toggle::On, &mut changes, &state).unwrap();
    set_flag(Flag::Quiet, Toggle::Off, &mut changes, &state).unwrap();

    assert_eq!(
        queued_line(&changes).as_deref(),
        Some("loglevel=3 mitigations=off sysrq_always_enabled=1 splash")
    );
    assert_eq!(changes.len(), 1, "Plymouth is installed already");

    for flag in [Flag::MitigationsOff, Flag::SysRq, Flag::Splash] {
        set_flag(flag, Toggle::Off, &mut changes, &state).unwrap();
    }

    set_flag(Flag::Quiet, Toggle::On, &mut changes, &state).unwrap();
    assert!(
        changes.is_empty(),
        "the file says `loglevel=3 quiet` already"
    );
}

#[test]
fn the_splash_queues_plymouth_while_it_is_missing() {
    let state = system(ARCH, &[]);
    let mut changes = ChangeSet::default();

    set_flag(Flag::Splash, Toggle::On, &mut changes, &state).unwrap();
    assert!(plymouth_queued(&changes));
    assert_eq!(changes.len(), 2);

    set_flag(Flag::Splash, Toggle::Off, &mut changes, &state).unwrap();
    assert!(!plymouth_queued(&changes));
    assert!(changes.is_empty());
}

#[test]
fn switching_the_splash_off_never_removes_an_installed_plymouth() {
    let state = system(
        "GRUB_CMDLINE_LINUX_DEFAULT=\"quiet splash\"\n",
        &["plymouth"],
    );

    let mut changes = ChangeSet::default();

    set_flag(Flag::Splash, Toggle::Off, &mut changes, &state).unwrap();

    assert_eq!(queued_line(&changes).as_deref(), Some("quiet"));
    assert_eq!(changes.wish::<Packages>(&package("plymouth")), None);
}

/// A command line the page could not write back word for word stays as
/// the administrator wrote it.
#[test]
fn a_command_line_with_quotes_is_left_alone() {
    let state = system(
        "GRUB_CMDLINE_LINUX_DEFAULT='quiet acpi_osi=\"Windows 2015\"'\n",
        &[],
    );

    let mut changes = ChangeSet::default();

    assert_eq!(shown(&state, &changes).unwrap().parameters, None);
    assert_eq!(
        add_parameter(parameter("splash"), &mut changes, &state),
        Err(Refusal::CustomCommandLine)
    );
    assert_eq!(
        set_flag(Flag::Quiet, Toggle::Off, &mut changes, &state),
        Err(Refusal::CustomCommandLine)
    );
    assert_eq!(
        remove_parameter("quiet", &mut changes, &state),
        Err(Refusal::CustomCommandLine)
    );
    assert!(changes.is_empty());
}

#[test]
fn a_full_command_line_takes_no_more() {
    let many: Vec<String> = (0_u32..128_u32)
        .map(|number| format!("p{number}"))
        .collect();

    let state = system(
        &format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"\n", many.join(" ")),
        &[],
    );

    let mut changes = ChangeSet::default();

    assert_eq!(
        add_parameter(parameter("one-more"), &mut changes, &state),
        Err(Refusal::TooManyParameters)
    );
    assert!(changes.is_empty());
}

#[test]
fn remembering_sets_both_variables_and_forgetting_names_an_entry_again() {
    let state = system(ARCH, &[]);
    let mut changes = ChangeSet::default();

    set_remember(true, entry("fallback"), &mut changes, &state).unwrap();
    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::Default),
        Some(GrubSetting::Default(DefaultEntry::Saved))
    );
    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::SaveDefault),
        Some(GrubSetting::SaveDefault(true))
    );
    assert!(shown(&state, &changes).unwrap().remember);

    set_remember(false, entry("fallback"), &mut changes, &state).unwrap();
    assert!(changes.is_empty(), "the file names entry 0 already");

    let saved = system("GRUB_DEFAULT=saved\nGRUB_SAVEDEFAULT=true\n", &[]);

    assert!(shown(&saved, &changes).unwrap().remember);
    set_remember(false, entry("gnulinux-simple-1"), &mut changes, &saved).unwrap();
    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::Default),
        Some(GrubSetting::Default(DefaultEntry::Entry(entry(
            "gnulinux-simple-1"
        ))))
    );
    assert_eq!(
        GrubDefaults::queued(&changes, GrubKey::SaveDefault),
        Some(GrubSetting::SaveDefault(false))
    );
}

#[test]
fn only_an_installed_plymouth_theme_can_be_chosen() {
    let without = system(ARCH, &[]);
    let with = with_plymouth(system(ARCH, &["plymouth"]));
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_plymouth_theme(plymouth_theme("spinner"), &mut changes, &without),
        Err(Refusal::UnknownTheme)
    );
    assert_eq!(
        set_plymouth_theme(plymouth_theme("no-such-theme"), &mut changes, &with),
        Err(Refusal::UnknownTheme)
    );
    assert!(changes.is_empty());

    set_plymouth_theme(plymouth_theme("spinner"), &mut changes, &with).unwrap();
    assert_eq!(
        changes.wish::<PlymouthThemes>(&Single),
        Some(&plymouth_theme("spinner"))
    );

    set_plymouth_theme(plymouth_theme("bgrt"), &mut changes, &with).unwrap();
    assert!(changes.is_empty(), "the theme in use is no change");
}

#[test]
fn clearing_picture_and_theme_queues_only_when_one_is_set() {
    let state = system("GRUB_THEME=\"/usr/share/grub/themes/x/theme.txt\"\n", &[]);
    let mut changes = ChangeSet::default();

    set(GrubSetting::Background(None), &mut changes, &state).unwrap();
    assert!(changes.is_empty());

    set(GrubSetting::Theme(None), &mut changes, &state).unwrap();
    assert_eq!(shown(&state, &changes).unwrap().theme, "");
    assert_eq!(changes.len(), 1);
}

#[test]
fn the_splash_package_is_plymouth() {
    assert_eq!(super::PLYMOUTH_PACKAGE.as_str(), "plymouth");
}

#[test]
fn every_switch_has_its_kernel_parameter() {
    for flag in Flag::ALL {
        assert_eq!(flag.kernel_parameter().as_str(), flag.parameter());
    }
}
