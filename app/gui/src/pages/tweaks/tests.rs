use std::path::Path;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Snapshot;
use toolbox_core::domains::{CurrentDns, CurrentWifiBackend};
use toolbox_plan::{DnsProvider, ShellPath, WifiBackend};

use super::TweaksPage;
use crate::catalogs::Catalogs;
use crate::testing::{PageFixture, assert_rows_wait_while_busy};

const STORE_FISH: &str = "/nix/store/rdxc7wg0r196bf6cicanfgcdkqhq9m9l-fish-4.9/bin/fish";
const STORE_BASH: &str =
    "/nix/store/4bj2kxdm1462fzcc2i2s4dn33g2angcc-bash-interactive-5.3p3/bin/bash";

fn shipped() -> PageFixture {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentDns(DnsProvider::Default));
    snapshot.insert(CurrentWifiBackend(WifiBackend::Iwd));

    PageFixture {
        catalogs: Catalogs::load(&dir),
        snapshot: snapshot.into(),
        ..PageFixture::default()
    }
}

#[test]
fn while_a_plan_runs_the_entries_wait_and_a_banner_says_why() {
    assert_rows_wait_while_busy(shipped, TweaksPage::default);
}

#[test]
fn while_a_plan_runs_the_dropdowns_wait_too() {
    for busy in [true, false] {
        let mut harness = PageFixture { busy, ..shipped() }.harness(TweaksPage::default());

        harness.run();

        let resting: Vec<bool> = harness
            .query_all_by_role(Role::ComboBox)
            .map(|dropdown| dropdown.accesskit_node().is_disabled())
            .collect();

        let waiting = resting.iter().filter(|resting| **resting).count();

        // The login shell stays unknown without a user, so it always rests.
        assert_eq!(
            waiting,
            if busy { 3 } else { 1 },
            "busy {busy}: {resting:?}"
        );
    }
}

fn with_shells(current: ShellPath, shells: Vec<ShellPath>) -> PageFixture {
    use std::collections::BTreeMap;

    use toolbox_core::domains::{AvailableShells, CurrentShells};
    use toolbox_plan::UserName;

    let user = UserName::new("tester").unwrap();
    let mut snapshot = Snapshot::default();
    snapshot.insert(toolbox_system::Host {
        user: Some(user.clone()),
        ..toolbox_system::Host::default()
    });
    snapshot.insert(CurrentShells(BTreeMap::from([(user, current)])));
    snapshot.insert(AvailableShells(shells));

    PageFixture {
        snapshot: snapshot.into(),
        ..shipped()
    }
}

fn with_fish() -> PageFixture {
    let fish = ShellPath::new("/usr/bin/fish").unwrap();
    let bash = ShellPath::new("/usr/bin/bash").unwrap();
    with_shells(fish.clone(), vec![bash, fish])
}

#[test]
fn the_login_shell_shows_its_name_and_tells_its_path() {
    let mut harness = with_fish().harness(TweaksPage::default());
    harness.run();

    let shell = harness.get_by_role_and_label(Role::ComboBox, "Login shell");
    let node = shell.accesskit_node();

    assert_eq!(node.value().as_deref(), Some("fish"));
    assert_eq!(node.description().as_deref(), Some("/usr/bin/fish"));
}

#[test]
fn every_section_counts_what_is_on() {
    let mut harness = shipped().harness(TweaksPage::default());
    harness.run();

    let heading = harness
        .get_by_role_and_label(Role::Heading, "Shell and network")
        .rect();

    let beside = harness
        .query_all_by_label("0/4")
        .any(|count| (count.rect().center().y - heading.center().y).abs() < heading.height());

    assert!(beside, "Shell and network does not count what is on");
    assert!(harness.query_by_label("4 settings").is_none());
}

/// A NixOS user's shells: each program twice, once in the store and
/// once on the system's path, so the list shows whole paths.
fn with_store_shells() -> PageFixture {
    let shells: Vec<ShellPath> = [
        STORE_BASH,
        "/run/current-system/sw/bin/bash",
        STORE_FISH,
        "/run/current-system/sw/bin/fish",
    ]
    .into_iter()
    .map(|path| ShellPath::new(path).unwrap())
    .collect();

    with_shells(ShellPath::new(STORE_FISH).unwrap(), shells)
}

#[test]
fn a_store_path_in_the_shell_list_is_cut_inside_a_tall_centred_entry() {
    use toolbox_gui_kit::testing::{blurred_rects, cap_off_centre, painted_text_within};
    use toolbox_gui_kit::theme::PILL_HEIGHT;

    let mut harness = with_store_shells().harness(TweaksPage::default());
    harness.run();
    harness
        .get_by_role_and_label(Role::ComboBox, "Login shell")
        .click();
    harness.run();

    let entries: Vec<(String, egui::Rect)> = harness
        .query_all_by_role(Role::ListBoxOption)
        .map(|entry| {
            (
                entry.accesskit_node().label().unwrap_or_default(),
                entry.rect(),
            )
        })
        .collect();

    assert_eq!(entries.len(), 4, "{entries:?}");
    let (_, first) = entries[0].clone();
    let popup = blurred_rects(&harness)
        .into_iter()
        .filter(|panel| panel.contains_rect(first))
        .min_by(|one, other| one.area().total_cmp(&other.area()))
        .expect("the list is painted on a panel");

    for (path, entry) in entries {
        assert!(
            popup.contains_rect(entry),
            "{path}: {entry:?} reaches past the list's {popup:?}"
        );
        assert!(entry.height() >= PILL_HEIGHT - 0.5, "{path}: {entry:?}");

        let text = painted_text_within(&harness, &path, entry).expect("the entry is painted");
        let right = text.pos.x + text.galley.rect.right();
        assert!(
            right <= entry.right(),
            "{path} ends at {right}, past {entry:?}"
        );
        if path.starts_with("/nix/store") {
            assert!(text.galley.elided, "{path} is not cut");
        }

        let off = cap_off_centre(&harness, &path, entry).expect("the text is painted");
        assert!(
            off.abs() < 0.75,
            "{path} sits {off} below its entry's middle"
        );
    }
}

#[test]
fn every_chip_counts_what_is_on() {
    let mut harness = shipped().harness(TweaksPage::default());
    harness.run();

    for chip in ["Shell and network 0/4", "All"] {
        assert!(
            harness
                .query_all_by_role(Role::Button)
                .filter_map(|button| button.accesskit_node().label())
                .any(|label| label.starts_with(chip) && label.contains('/')),
            "{chip} has no count"
        );
    }
}
