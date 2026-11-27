//! The steps each root action runs, in the order the helper runs them.

use super::packages::{
    install, install_built, install_dependencies, refresh_keyring, remote, remove,
    remove_make_dependencies, remove_orphans, upgrade,
};
use super::{RootAction, accounts, boot, network, on_units, repair, run, settings};
use crate::Mirrorlist;
use crate::procedure::{Builtin, Command, Invocation, Program};

impl RootAction {
    #[expect(
        clippy::too_many_lines,
        reason = "one arm per action, a new action fails to compile until it names its steps"
    )]
    pub fn procedure(&self) -> Vec<Invocation> {
        match self {
            Self::InstallPackages { packages } => vec![run(install(packages))],
            Self::RemovePackages { packages } => vec![remove(packages)],
            Self::InstallDependencies { packages } => vec![install_dependencies(packages)],
            Self::InstallBuilt {
                run: built,
                packages,
                reason,
            } => vec![install_built(built, packages, *reason)],
            Self::UpgradeSystem { upgrader } => vec![run(upgrade(*upgrader))],
            Self::CleanPackageCache => vec![run(Command::new(Program::Paccache).arg("-ruk0"))],
            Self::RemoveOrphans => vec![remove_orphans()],
            Self::RemoveMakeDependencies { packages } => {
                vec![remove_make_dependencies(packages)]
            }
            Self::RefreshKeyring => vec![run(refresh_keyring())],
            Self::RemoveDatabaseLock => vec![Invocation::Builtin(Builtin::RemoveDatabaseLock)],
            Self::GarudaUpdateRemote { action } => vec![run(remote(*action))],
            Self::GarudaHealthFix => vec![run(Command::new(Program::GarudaHealth).arg("--fix"))],
            Self::ChrootInstallation => vec![run(Command::new(Program::GarudaChroot).arg("-a"))],
            Self::InstallMirrorlist { mirrorlist } => install_mirrorlist(mirrorlist),
            Self::EnableUnits { units } => on_units(["enable", "--now"], units),
            Self::DisableUnits { units } => on_units(["disable", "--now"], units),
            Self::ControlUnits { operation, units } => on_units([operation.as_str()], units),
            Self::MaskUnits { units } => on_units(["mask"], units),
            Self::UnmaskUnits { units } => on_units(["unmask"], units),
            Self::AddToGroup { user, group } => vec![run(accounts::gpasswd("-a", user, group))],
            Self::RemoveFromGroup { user, group } => {
                vec![run(accounts::gpasswd("-d", user, group))]
            }
            Self::SetShell { user, shell } => settings::set_shell(user, shell),
            Self::SetDns { provider } => settings::set_dns(*provider),
            Self::SetHblock { enabled } => settings::set_hblock(*enabled),
            Self::SetWifiBackend { backend } => settings::set_wifi_backend(*backend),
            Self::SetGarudaUpdateFlag { flag, enabled } => {
                settings::set_garuda_update_flag(*flag, *enabled)
            }
            Self::SetLocaleEnabled { locale, enabled } => {
                settings::set_locale_enabled(locale, *enabled)
            }
            Self::GenerateLocales => vec![run(Command::new(Program::LocaleGen))],
            Self::SetSystemLocale { locale } => settings::set_system_locale(locale),
            Self::InstallDkmsModule { build } => vec![run(build.command())],
            Self::CreateUser { user } => accounts::create_user(user),
            Self::SetPassword { user, password } => accounts::set_password(user, password.as_ref()),
            Self::DeleteUser { user, home } => accounts::delete_user(user, *home),
            Self::SetUserLocked { user, locked } => accounts::set_locked(user, *locked),
            Self::SetFullName { user, full_name } => accounts::set_full_name(user, full_name),
            Self::SetHomeFolder { user, change } => accounts::set_home_folder(user, change),
            Self::CreateGroup { group } => accounts::create_group(group),
            Self::DeleteGroup { group } => accounts::delete_group(group),
            Self::WriteConfigFile { write } => vec![write.invocation()],
            Self::ResolveMerge { merge } => {
                vec![Invocation::Builtin(Builtin::ResolveMerge(merge.clone()))]
            }
            Self::SnapshotBeforeMerges => vec![Invocation::Builtin(Builtin::SnapshotBeforeMerges)],
            Self::LoadModule { module } => network::load_module(module),
            Self::UnloadModule { module } => network::unload_module(module),
            Self::SetModuleBlacklisted {
                module,
                blacklisted,
            } => network::set_module_blacklisted(module, *blacklisted),
            Self::SetGrubSettings { settings } => boot::set_grub_settings(settings),
            Self::SetLoaderSettings { settings } => boot::set_loader_settings(settings),
            Self::GenerateGrubConfig => boot::generate_grub_config(),
            Self::SetPlymouthTheme { theme } => boot::set_plymouth_theme(theme),
            Self::SetKernelCmdline { parameters } => boot::set_kernel_cmdline(parameters),
            Self::SelectPlymouthTheme { theme } => vec![boot::plymouth_theme(theme)],
            Self::RebuildUkis { initramfs } => boot::rebuild_ukis(*initramfs),
            Self::RepairBoot {
                system,
                work,
                passphrase,
            } => repair::repair_boot(system, work, passphrase.as_ref()),
        }
    }
}

fn install_mirrorlist(mirrorlist: &Mirrorlist) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::InstallMirrorlist(
        mirrorlist.clone(),
    ))]
}
