mod risky;

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::domains::{CurrentGrub, InstalledPackages};
use toolbox_core::features::boot::Flag;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState, PackageName};
use toolbox_system::{Host, Menu, MenuRead};

use super::BootPage;
use crate::boot::{Boot, BootRefusal, FileKind};
use crate::command::{BootCommand, Command};
use crate::testing::captures::GRUB_CFG;
use crate::testing::{PageFixture, PageState};

/// `/etc/default/grub` of a Garuda Mokka install, the install of
/// [`GRUB_CFG`]. The file names the Catppuccin theme.
const DEFAULTS: &str =
    include_str!("../../../../../crates/plan/src/grub/fixtures/installed/default-grub");
const THEME: &str = "/usr/share/grub/themes/catppuccin-mocha/theme.txt";
const MASKED: &str = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx";

pub(super) type Shown = Harness<'static, PageState<BootPage>>;

pub(super) fn grub_system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Host::default());
    snapshot.insert(CurrentGrub(GrubState::read(DEFAULTS)));
    snapshot.insert(InstalledPackages::default());
    snapshot
}

pub(super) fn menu() -> Menu {
    Menu::Read(MenuRead::Known(GrubMenu::for_platform(
        GRUB_CFG,
        GrubPlatform::Efi,
    )))
}

pub(super) fn page(snapshot: Snapshot, boot: Boot) -> Shown {
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        boot,
        ..PageFixture::default()
    }
    .harness(BootPage::default());

    harness.run();
    harness
}

fn ready() -> Shown {
    page(grub_system(), Boot::showing(menu(), None, None))
}

fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

fn edit(change: Edit) -> Command {
    Command::from(BootCommand::Edit(change))
}

#[test]
fn a_system_without_grub_says_so() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Host::default());
    let harness = page(snapshot, Boot::default());

    assert!(
        harness
            .query_by_label("GRUB is not the boot loader of this system")
            .is_some()
    );
}

#[test]
fn a_picked_entry_becomes_the_default() {
    let mut harness = ready();

    harness
        .get_by_role_and_label(Role::ComboBox, "Default entry")
        .click();
    harness.run();
    harness
        .get_by_label(
            "Garuda Linux, with Linux linux-garuda (in Advanced options for Garuda Linux)",
        )
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [edit(Edit::Default(format!(
            "gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-advanced-{MASKED}"
        )))]
    );
}

#[test]
fn the_timeout_steps_by_a_second_and_switches_flip() {
    let mut harness = ready();

    harness.get_by_label("One second more").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "Group entries in submenus")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [edit(Edit::Timeout(6)), edit(Edit::Submenus(false))]
    );
}

#[test]
fn the_timeout_is_a_pill_field_that_takes_typed_seconds() {
    let mut harness = ready();
    let name = "Menu timeout in seconds";
    let field = harness.get_by_role_and_label(Role::TextInput, name).rect();
    let unit = toolbox_gui_kit::testing::painted_text_within(&harness, "s", field);

    assert!((field.height() - toolbox_gui_kit::theme::PILL_HEIGHT).abs() < 0.5);
    assert!(unit.is_some(), "no unit inside {field:?}");

    harness.get_by_label(name).click();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    harness.get_by_label(name).type_text("12");
    harness.key_press(egui::Key::Enter);
    harness.run();

    assert_eq!(sent(&harness), [edit(Edit::Timeout(12))]);
}

#[test]
fn parameters_are_added_and_removed() {
    let mut harness = ready();

    // The kernel card lies below the window at first.
    harness.get_by_label("Remove loglevel=3").scroll_to_me();
    harness.run();
    harness.get_by_label("Remove loglevel=3").click();
    harness.run();
    harness.get_by_label("New kernel parameter").click();
    harness.run();
    harness
        .get_by_label("New kernel parameter")
        .type_text("nowatchdog");
    harness.run();
    harness.get_by_label("Add").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            edit(Edit::RemoveParameter("loglevel=3".to_owned())),
            edit(Edit::AddParameter {
                text: "nowatchdog".to_owned(),
                confirmed: false,
            }),
        ]
    );
}

#[test]
fn a_known_flag_is_a_switch_of_its_own() {
    let mut harness = ready();

    harness
        .get_by_role_and_label(Role::CheckBox, "Hide most boot messages (quiet)")
        .click();
    harness.run();

    assert_eq!(sent(&harness), [edit(Edit::Flag(Flag::Quiet, false))]);
}

#[test]
fn a_refused_value_is_named_until_dismissed() {
    let refusal = BootRefusal {
        code: "invalid-character",
        value: "a b".to_owned(),
    };

    let mut harness = page(grub_system(), Boot::showing(menu(), Some(refusal), None));

    assert!(
        harness
            .query_by_label_contains("a character that is not allowed here")
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(BootCommand::DismissRefusal)]);
}

#[test]
fn a_proposed_kernel_is_queued_as_the_default_on_request() {
    let kernel = PackageName::new("linux-garuda").unwrap();
    let mut harness = page(grub_system(), Boot::showing(menu(), None, Some(kernel)));

    assert!(
        harness
            .query_by_label_contains("The menu entry of the kernel linux-garuda can become")
            .is_some()
    );

    harness.get_by_label("Queue as default").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            edit(Edit::Default(format!(
                "gnulinux-advanced-{MASKED}>gnulinux-linux-garuda-advanced-{MASKED}"
            ))),
            Command::from(BootCommand::DropProposal),
        ]
    );
}

#[test]
fn choose_asks_for_the_file_of_its_row() {
    let mut harness = ready();

    harness.get_by_label("Choose GRUB theme").scroll_to_me();
    harness.run();
    harness.get_by_label("Choose GRUB theme").click();
    harness.run();

    assert!(matches!(
        sent(&harness).as_slice(),
        [Command::App(crate::command::AppCommand::Boot(
            BootCommand::Choose {
                kind: FileKind::Theme,
                title: _,
                filter: _,
            }
        ))]
    ));
}

#[test]
fn a_queued_change_is_marked() {
    let snapshot = grub_system();
    let mut changes = ChangeSet::default();
    toolbox_core::features::boot::edit::edit(Edit::Timeout(9), &mut changes, &snapshot).unwrap();

    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        changes,
        boot: Boot::showing(menu(), None, None),
        ..PageFixture::default()
    }
    .harness(BootPage::default());

    harness.run();

    assert!(harness.query_by_label("Changes with Apply").is_some());
}

#[test]
fn use_none_waits_in_the_menu_of_its_row() {
    let mut harness = ready();

    assert!(harness.query_by_label("Use none").is_none());
    harness
        .get_by_label("More actions for GRUB theme")
        .scroll_to_me();
    harness.run();
    harness.get_by_label("More actions for GRUB theme").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::MenuItem, "Use none")
        .click();
    harness.run();

    assert_eq!(sent(&harness), [edit(Edit::Theme(String::new()))]);
}

#[test]
fn the_new_parameter_field_is_as_tall_as_add_and_centred_on_it() {
    let mut harness = ready();
    harness.get_by_label("New kernel parameter").scroll_to_me();
    harness.run();

    let field = harness.get_by_label("New kernel parameter").rect();
    let add = harness.get_by_label("Add").rect();

    assert!(
        (field.height() - add.height()).abs() < 0.5,
        "the field is {} tall, Add {}",
        field.height(),
        add.height()
    );
    assert!(
        (field.center().y - add.center().y).abs() < 0.5,
        "the field's middle is at {}, Add's at {}",
        field.center().y,
        add.center().y
    );
}

#[test]
fn a_file_rows_description_uses_the_width_up_to_its_controls() {
    let mut harness = ready();
    harness
        .get_by_label("Choose GRUB background picture")
        .scroll_to_me();
    harness.run();

    let description = harness
        .get_by_label(
            "A PNG, JPEG or TGA picture behind the boot menu. A GRUB theme has its own background.",
        )
        .rect();

    let choose = harness
        .get_by_label("Choose GRUB background picture")
        .rect();

    let line = harness.ctx.global_style().text_styles[&egui::TextStyle::Body].size * 2.0;

    assert!(
        description.height() < line,
        "the description wraps onto {} px though {} px are free beside it",
        description.height(),
        choose.left() - description.left()
    );
    assert!(
        description.right() < choose.left(),
        "{description:?} {choose:?}"
    );
}

#[test]
fn the_new_parameter_hint_is_centred_in_its_field() {
    let mut harness = ready();
    harness.get_by_label("New kernel parameter").scroll_to_me();
    harness.run();

    let field = harness.get_by_label("New kernel parameter").rect();
    let off = toolbox_gui_kit::testing::cap_off_centre(&harness, "New parameter", field)
        .expect("the hint is painted");

    assert!(
        off.abs() < 0.75,
        "the hint sits {off} off the field's middle"
    );
}

#[test]
fn a_long_chosen_path_sits_under_the_words_and_leaves_choose_in_line() {
    let mut harness = ready();
    harness
        .get_by_label("More actions for GRUB theme")
        .scroll_to_me();
    harness.run();

    let chooses = [
        harness
            .get_by_label("Choose GRUB background picture")
            .rect(),
        harness.get_by_label("Choose GRUB theme").rect(),
    ];

    assert!(
        (chooses[0].right() - chooses[1].right()).abs() < 0.5,
        "the Choose buttons do not line up: {chooses:?}"
    );

    let path = harness.get_by_label_contains(THEME).rect();

    assert!(
        path.right() <= chooses[1].left(),
        "the path {path:?} runs into Choose {:?}",
        chooses[1]
    );
}
