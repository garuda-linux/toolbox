use toolbox_plan::LoaderDefault;
use toolbox_plan::sdboot::{LoaderKey, LoaderSetting, LoaderSettings, LoaderTimeout};

use super::{CurrentLoader, LoaderSettingsDomain};
use crate::test_support::commands;
use crate::{ChangeSet, Snapshot, plan};

/// `/efi/loader/loader.conf` of the Garuda Lima image.
const IMAGE: &str = toolbox_plan::testing::sdboot::LOADER_CONF;

fn image() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(CurrentLoader(LoaderSettings::parse(IMAGE)));
    snapshot
}

fn seconds(value: u32) -> LoaderSetting {
    LoaderSetting::Timeout(Some(LoaderTimeout::Seconds(value)))
}

/// The image's file names no setting, so asking for systemd-boot's own
/// default changes nothing.
#[test]
fn a_wish_the_file_already_fulfils_drops_out() {
    let state = image();
    let mut changes = ChangeSet::default();

    LoaderSettingsDomain::want(&mut changes, LoaderSetting::Timeout(None), &state);
    LoaderSettingsDomain::want(&mut changes, LoaderSetting::Editor(None), &state);

    assert!(changes.is_empty());
}

#[test]
fn a_second_wish_replaces_the_first_and_can_take_it_back() {
    let state = image();
    let mut changes = ChangeSet::default();

    LoaderSettingsDomain::want(&mut changes, seconds(5), &state);
    LoaderSettingsDomain::want(&mut changes, seconds(9), &state);
    assert_eq!(
        LoaderSettingsDomain::queued(&changes, LoaderKey::Timeout),
        Some(seconds(9))
    );

    LoaderSettingsDomain::want(&mut changes, LoaderSetting::Timeout(None), &state);
    assert!(changes.is_empty());
}

/// systemd-boot reads the file at each start, so nothing is built after.
#[test]
fn many_settings_are_one_edit_and_nothing_else() {
    let state = image();
    let mut changes = ChangeSet::default();

    LoaderSettingsDomain::want(&mut changes, seconds(3), &state);
    LoaderSettingsDomain::want(
        &mut changes,
        LoaderSetting::Default(Some(LoaderDefault::new("garuda-*-garuda.efi").unwrap())),
        &state,
    );
    LoaderSettingsDomain::want(&mut changes, LoaderSetting::Editor(Some(false)), &state);

    assert_eq!(
        commands(&plan(&changes)),
        [
            "set timeout 3, default garuda-*-garuda.efi, editor no in loader/loader.conf on the EFI system partition"
        ]
    );
}
