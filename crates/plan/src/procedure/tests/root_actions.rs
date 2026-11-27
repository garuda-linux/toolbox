use std::collections::BTreeSet;

use crate::sdboot::{LoaderChange, LoaderSetting, LoaderTimeout};
use crate::{
    BootloaderId, BuildRun, ConfigFilePath, ConfigText, ConfigWrite, DiskDevice, DkmsBuild,
    DkmsModuleName, DkmsVersion, DnsProvider, Fingerprint, FullName, GarudaUpdateFlag, GroupName,
    GrubSetting, GrubSettings, HomeChange, HomeDirectory, HomeFolder, InstallReason,
    InstalledSystem, KernelModule, KernelParameters, KernelRelease, LocaleEntry, LocaleName,
    MergeChoice, MergeCopy, MergeResolve, Mirrorlist, NewHome, NewUser, NewUserParts, PackageName,
    PartitionDevice, Password, PlymouthTheme, RemoteAction, RepairRoot, RepairWork, Resolution,
    RootAction, RootLayout, ShellPath, TargetList, UnitName, UnitOperation, Upgrader, UserName,
    VolumeDevice, WifiBackend,
};

/// Every kind [`kind`] names. Written by hand, so a variant missing here
/// fails [`every_root_action_variant_has_a_sample`].
const KINDS: [&str; 52] = [
    "install-packages",
    "remove-packages",
    "install-dependencies",
    "install-built",
    "upgrade-system",
    "clean-package-cache",
    "remove-orphans",
    "remove-make-dependencies",
    "refresh-keyring",
    "remove-database-lock",
    "garuda-update-remote",
    "garuda-health-fix",
    "chroot-installation",
    "install-mirrorlist",
    "enable-units",
    "disable-units",
    "control-units",
    "mask-units",
    "unmask-units",
    "add-to-group",
    "remove-from-group",
    "set-shell",
    "set-dns",
    "set-hblock",
    "set-wifi-backend",
    "set-garuda-update-flag",
    "set-locale-enabled",
    "generate-locales",
    "set-system-locale",
    "install-dkms-module",
    "create-user",
    "set-password",
    "delete-user",
    "set-user-locked",
    "set-full-name",
    "set-home-folder",
    "create-group",
    "delete-group",
    "write-config-file",
    "resolve-merge",
    "snapshot-before-merges",
    "load-module",
    "unload-module",
    "set-module-blacklisted",
    "set-grub-settings",
    "set-loader-settings",
    "generate-grub-config",
    "set-plymouth-theme",
    "set-kernel-cmdline",
    "select-plymouth-theme",
    "rebuild-ukis",
    "repair-boot",
];

/// The wire `kind` of every variant. It has no wildcard arm, so a new
/// variant does not compile until it is named here, and then [`KINDS`] and
/// [`every_root_action`] must cover it too.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per variant, so a new variant fails to compile until it has a name"
)]
fn kind(action: &RootAction) -> &'static str {
    match action {
        RootAction::InstallPackages { packages: _ } => "install-packages",
        RootAction::RemovePackages { packages: _ } => "remove-packages",
        RootAction::InstallDependencies { packages: _ } => "install-dependencies",
        RootAction::InstallBuilt {
            run: _,
            packages: _,
            reason: _,
        } => "install-built",
        RootAction::UpgradeSystem { upgrader: _ } => "upgrade-system",
        RootAction::CleanPackageCache => "clean-package-cache",
        RootAction::RemoveOrphans => "remove-orphans",
        RootAction::RemoveMakeDependencies { packages: _ } => "remove-make-dependencies",
        RootAction::RefreshKeyring => "refresh-keyring",
        RootAction::RemoveDatabaseLock => "remove-database-lock",
        RootAction::GarudaUpdateRemote { action: _ } => "garuda-update-remote",
        RootAction::GarudaHealthFix => "garuda-health-fix",
        RootAction::ChrootInstallation => "chroot-installation",
        RootAction::InstallMirrorlist { mirrorlist: _ } => "install-mirrorlist",
        RootAction::EnableUnits { units: _ } => "enable-units",
        RootAction::DisableUnits { units: _ } => "disable-units",
        RootAction::ControlUnits {
            operation: _,
            units: _,
        } => "control-units",
        RootAction::MaskUnits { units: _ } => "mask-units",
        RootAction::UnmaskUnits { units: _ } => "unmask-units",
        RootAction::AddToGroup { user: _, group: _ } => "add-to-group",
        RootAction::RemoveFromGroup { user: _, group: _ } => "remove-from-group",
        RootAction::SetShell { user: _, shell: _ } => "set-shell",
        RootAction::SetDns { provider: _ } => "set-dns",
        RootAction::SetHblock { enabled: _ } => "set-hblock",
        RootAction::SetWifiBackend { backend: _ } => "set-wifi-backend",
        RootAction::SetGarudaUpdateFlag {
            flag: _,
            enabled: _,
        } => "set-garuda-update-flag",
        RootAction::SetLocaleEnabled {
            locale: _,
            enabled: _,
        } => "set-locale-enabled",
        RootAction::GenerateLocales => "generate-locales",
        RootAction::SetSystemLocale { locale: _ } => "set-system-locale",
        RootAction::InstallDkmsModule { build: _ } => "install-dkms-module",
        RootAction::CreateUser { user: _ } => "create-user",
        RootAction::SetPassword {
            user: _,
            password: _,
        } => "set-password",
        RootAction::DeleteUser { user: _, home: _ } => "delete-user",
        RootAction::SetUserLocked { user: _, locked: _ } => "set-user-locked",
        RootAction::SetFullName {
            user: _,
            full_name: _,
        } => "set-full-name",
        RootAction::SetHomeFolder { user: _, change: _ } => "set-home-folder",
        RootAction::CreateGroup { group: _ } => "create-group",
        RootAction::DeleteGroup { group: _ } => "delete-group",
        RootAction::WriteConfigFile { write: _ } => "write-config-file",
        RootAction::ResolveMerge { merge: _ } => "resolve-merge",
        RootAction::SnapshotBeforeMerges => "snapshot-before-merges",
        RootAction::LoadModule { module: _ } => "load-module",
        RootAction::UnloadModule { module: _ } => "unload-module",
        RootAction::SetModuleBlacklisted {
            module: _,
            blacklisted: _,
        } => "set-module-blacklisted",
        RootAction::SetGrubSettings { settings: _ } => "set-grub-settings",
        RootAction::SetLoaderSettings { settings: _ } => "set-loader-settings",
        RootAction::GenerateGrubConfig => "generate-grub-config",
        RootAction::SetPlymouthTheme { theme: _ } => "set-plymouth-theme",
        RootAction::SetKernelCmdline { parameters: _ } => "set-kernel-cmdline",
        RootAction::SelectPlymouthTheme { theme: _ } => "select-plymouth-theme",
        RootAction::RebuildUkis { initramfs: _ } => "rebuild-ukis",
        RootAction::RepairBoot {
            system: _,
            work: _,
            passphrase: _,
        } => "repair-boot",
    }
}

fn password() -> Password {
    Password::new("correct horse: battery staple".to_owned()).unwrap()
}

/// One sample per variant, with the simplest valid values.
#[expect(
    clippy::too_many_lines,
    reason = "one sample per variant, so a new variant adds its sample here"
)]
pub(super) fn every_root_action() -> Vec<RootAction> {
    let user = || UserName::new("ada").unwrap();
    let group = || GroupName::new("docker").unwrap();
    let units = || TargetList::new([UnitName::new("cups.socket").unwrap()]).unwrap();
    let packages = || TargetList::new([PackageName::new("mpv").unwrap()]).unwrap();
    let module = || KernelModule::new("nvidia").unwrap();
    let theme = || PlymouthTheme::new("bgrt").unwrap();

    let new_user = NewUser::new(NewUserParts {
        name: UserName::new("guest").unwrap(),
        full_name: FullName::new("Guest").unwrap(),
        shell: None,
        groups: BTreeSet::from([GroupName::new("wheel").unwrap()]),
        home: NewHome::Standard,
        uid: None,
        system: false,
    })
    .unwrap();

    let closed = InstalledSystem {
        root: RepairRoot::Open {
            device: VolumeDevice::new("/dev/sda2").unwrap(),
        },
        layout: RootLayout::Plain,
    };

    let locked = InstalledSystem {
        root: RepairRoot::Locked {
            partition: PartitionDevice::new("/dev/sda3").unwrap(),
        },
        layout: RootLayout::Subvolume,
    };

    vec![
        RootAction::InstallPackages {
            packages: packages(),
        },
        RootAction::RemovePackages {
            packages: packages(),
        },
        RootAction::InstallDependencies {
            packages: packages(),
        },
        RootAction::InstallBuilt {
            run: BuildRun::new("1a2b").unwrap(),
            packages: packages(),
            reason: InstallReason::Explicit,
        },
        RootAction::UpgradeSystem {
            upgrader: Upgrader::Pacman,
        },
        RootAction::UpgradeSystem {
            upgrader: Upgrader::GarudaUpdate,
        },
        RootAction::CleanPackageCache,
        RootAction::RemoveOrphans,
        RootAction::RemoveMakeDependencies {
            packages: packages(),
        },
        RootAction::RefreshKeyring,
        RootAction::RemoveDatabaseLock,
        RootAction::GarudaUpdateRemote {
            action: RemoteAction::Fix,
        },
        RootAction::GarudaHealthFix,
        RootAction::ChrootInstallation,
        RootAction::InstallMirrorlist {
            mirrorlist: Mirrorlist::new("Server = https://cdn-mirror.chaotic.cx/$repo/$arch\n")
                .unwrap(),
        },
        RootAction::EnableUnits { units: units() },
        RootAction::DisableUnits { units: units() },
        RootAction::ControlUnits {
            operation: UnitOperation::Restart,
            units: units(),
        },
        RootAction::MaskUnits { units: units() },
        RootAction::UnmaskUnits { units: units() },
        RootAction::AddToGroup {
            user: user(),
            group: group(),
        },
        RootAction::RemoveFromGroup {
            user: user(),
            group: group(),
        },
        RootAction::SetShell {
            user: user(),
            shell: ShellPath::new("/bin/bash").unwrap(),
        },
        RootAction::SetDns {
            provider: DnsProvider::Quad9,
        },
        RootAction::SetHblock {
            enabled: crate::Toggle::On,
        },
        RootAction::SetWifiBackend {
            backend: WifiBackend::Iwd,
        },
        RootAction::SetGarudaUpdateFlag {
            flag: GarudaUpdateFlag::UpdateAur,
            enabled: crate::Toggle::Off,
        },
        RootAction::SetLocaleEnabled {
            locale: LocaleEntry::parse("de_DE.UTF-8 UTF-8").unwrap(),
            enabled: crate::Toggle::On,
        },
        RootAction::GenerateLocales,
        RootAction::SetSystemLocale {
            locale: LocaleName::new("de_DE.UTF-8").unwrap(),
        },
        RootAction::InstallDkmsModule {
            build: DkmsBuild {
                module: DkmsModuleName::new("vboxhost").unwrap(),
                version: DkmsVersion::new("7.1.6_OSE").unwrap(),
                kernel: KernelRelease::new("6.12.9-1-cachyos-lts").unwrap(),
            },
        },
        RootAction::CreateUser { user: new_user },
        RootAction::SetPassword {
            user: user(),
            password: Some(password()),
        },
        RootAction::DeleteUser {
            user: user(),
            home: HomeFolder::Delete,
        },
        RootAction::SetUserLocked {
            user: user(),
            locked: crate::Toggle::On,
        },
        RootAction::SetFullName {
            user: user(),
            full_name: FullName::new("Ada").unwrap(),
        },
        RootAction::SetHomeFolder {
            user: user(),
            change: HomeChange {
                home: HomeDirectory::new("/home/ada/moved").unwrap(),
                move_files: true,
            },
        },
        RootAction::CreateGroup { group: group() },
        RootAction::DeleteGroup { group: group() },
        RootAction::WriteConfigFile {
            write: ConfigWrite {
                path: ConfigFilePath::new("/etc/fstab").unwrap(),
                content: ConfigText::new("a = 1\n").unwrap(),
                original: Fingerprint::of(b"old"),
            },
        },
        RootAction::ResolveMerge {
            merge: MergeResolve {
                copy: MergeCopy::new("/etc/pacman.conf.pacnew").unwrap(),
                choice: MergeChoice {
                    resolution: Resolution::TakeCopy,
                    copy: Fingerprint::of(b"new"),
                    current: Some(Fingerprint::of(b"old")),
                },
            },
        },
        RootAction::SnapshotBeforeMerges,
        RootAction::LoadModule { module: module() },
        RootAction::UnloadModule { module: module() },
        RootAction::SetModuleBlacklisted {
            module: module(),
            blacklisted: crate::Toggle::On,
        },
        RootAction::SetGrubSettings {
            settings: GrubSettings::new(vec![GrubSetting::Submenus(true)]).unwrap(),
        },
        RootAction::SetLoaderSettings {
            settings: LoaderChange::new(vec![LoaderSetting::Timeout(Some(
                LoaderTimeout::Seconds(5),
            ))])
            .unwrap(),
        },
        RootAction::GenerateGrubConfig,
        RootAction::SetPlymouthTheme { theme: theme() },
        RootAction::SetKernelCmdline {
            parameters: KernelParameters::parse("quiet splash").unwrap(),
        },
        RootAction::SelectPlymouthTheme { theme: theme() },
        RootAction::RebuildUkis { initramfs: true },
        RootAction::RepairBoot {
            system: closed.clone(),
            work: RepairWork::InstallBios {
                disk: DiskDevice::new("/dev/sda").unwrap(),
            },
            passphrase: None,
        },
        RootAction::RepairBoot {
            system: closed,
            work: RepairWork::InstallUefi {
                esp: PartitionDevice::new("/dev/sda1").unwrap(),
                id: BootloaderId::new("Garuda").unwrap(),
            },
            passphrase: None,
        },
        RootAction::RepairBoot {
            system: locked,
            work: RepairWork::RegenerateConfig { esp: None },
            passphrase: Some(password()),
        },
    ]
}

#[test]
fn every_root_action_variant_has_a_sample() {
    let mut seen: Vec<&str> = every_root_action().iter().map(kind).collect();
    seen.sort_unstable();
    seen.dedup();

    let mut expected = KINDS.to_vec();
    expected.sort_unstable();

    assert_eq!(seen, expected);
}

/// The hand-written name of each sample is the `kind` serde writes for it,
/// the one the helper reads back.
#[test]
fn every_sample_is_written_under_its_kind_name() {
    for action in every_root_action() {
        let json = serde_json::to_value(&action).unwrap();

        assert_eq!(json["kind"], kind(&action), "{action:?}");
    }
}
