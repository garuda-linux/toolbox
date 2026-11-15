mod counts;
mod tiles;

use std::path::Path;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Snapshot;
use toolbox_core::domains::{CurrentDns, CurrentWifiBackend, HblockState};
use toolbox_plan::{DnsProvider, ShellPath, WifiBackend};
use toolbox_system::{FailedProbes, Probe};

use super::TweaksPage;
use crate::catalogs::Catalogs;
use crate::testing::{PageFixture, assert_rows_wait_while_busy};

const STORE_FISH: &str = "/nix/store/rdxc7wg0r196bf6cicanfgcdkqhq9m9l-fish-4.9/bin/fish";
const STORE_BASH: &str =
    "/nix/store/4bj2kxdm1462fzcc2i2s4dn33g2angcc-bash-interactive-5.3p3/bin/bash";

type Shown = egui_kittest::Harness<'static, crate::testing::PageState<TweaksPage>>;

fn shipped() -> PageFixture {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentDns(DnsProvider::Default));
    snapshot.insert(CurrentWifiBackend(WifiBackend::Iwd));
    snapshot.insert(HblockState(false));

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
fn a_frame_with_the_same_system_builds_no_choices() {
    let mut harness = shipped().harness(TweaksPage::default());
    harness.run();
    let built = harness.state().page.lead_builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.lead_builds(), built);

    harness.state_mut().fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.lead_builds(), built + 1);

    harness.state_mut().fixture.snapshot = with_fish().snapshot;
    harness.run();
    assert_eq!(harness.state().page.lead_builds(), built + 2);
    assert!(
        harness
            .query_by_role_and_label(Role::ComboBox, "Login shell")
            .is_some()
    );
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
        .query_all_by_label("0/1")
        .any(|count| (count.rect().center().y - heading.center().y).abs() < heading.height());

    assert!(beside, "Shell and network does not count its one switch");
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

    for chip in ["Shell and network 0/1", "All"] {
        assert!(
            harness
                .query_all_by_role(Role::Button)
                .filter_map(|button| button.accesskit_node().label())
                .any(|label| label.starts_with(chip) && label.contains('/')),
            "{chip} has no count"
        );
    }
}

/// Without a catalog, since the empty snapshot leaves every catalog row
/// unread and the banner would count them too.
fn probed(failed: &[Probe]) -> PageFixture {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentWifiBackend(WifiBackend::Iwd));
    snapshot.insert(FailedProbes::new(failed.iter().copied()));

    PageFixture {
        snapshot: snapshot.into(),
        catalogs: Catalogs::default(),
        ..PageFixture::default()
    }
}

fn dns_value(harness: &Shown) -> String {
    harness
        .get_by_role_and_label(Role::ComboBox, "DNS server")
        .accesskit_node()
        .value()
        .unwrap_or_default()
}

/// A failed probe may have missed a value the system has, so the picker
/// joins the banner of what could not be read.
#[test]
fn a_picker_whose_probe_failed_counts_into_the_banner_and_is_not_called_unavailable() {
    let mut harness = probed(&[Probe::Dns]).harness(TweaksPage::default());
    harness.run();

    assert_ne!(dns_value(&harness), "Not available on this system");
    assert!(
        harness
            .query_by_label_contains("The state of one entry could not be read")
            .is_some()
    );
}

#[test]
fn a_picker_the_system_lacks_says_so_without_a_banner() {
    let mut harness = probed(&[]).harness(TweaksPage::default());
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::ComboBox, "DNS server")
            .is_none(),
        "a dropdown that never opens"
    );
    assert!(
        harness
            .query_all_by_label("Not available on this system")
            .next()
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("The state of one entry could not be read")
            .is_none()
    );
}

#[test]
fn before_the_first_probe_no_picker_is_called_unavailable() {
    let mut harness = PageFixture {
        probed: false,
        snapshot: Snapshot::default().into(),
        ..shipped()
    }
    .harness(TweaksPage::default());

    harness.run();
    assert_ne!(dns_value(&harness), "Not available on this system");
}
