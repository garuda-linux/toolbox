mod accounts;
mod cmdline;
mod grub;
mod home_file;
mod loader;
mod locale;
mod modules;
mod settings;
#[cfg(test)]
mod tests;
mod user_config;

use std::fs::Metadata;
use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

pub use cmdline::write_kernel_cmdline;
pub use grub::edit_grub_defaults;
pub use home_file::write_home_file;
pub use locale::{SystemLocaleChange, edit_locale_gen, set_system_locale};
pub use modules::edit_modprobe_blacklist;
pub use settings::{
    FileChange, set_garuda_update_flag, shell_is_listed, write_dns_config, write_wifi_backend,
};
use toolbox_plan::procedure::{Builtin, CHAOTIC_MIRRORLIST, DATABASE_LOCK, SHELLS};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    DNS_CONFIG, GARUDA_UPDATE_CONFIG, GRUB_DEFAULTS, HomeWrite, LOCALE_CONF, LOCALE_GEN,
    LocaleName, MODPROBE_BLACKLIST, Mirrorlist, NETWORKMANAGER_CONF_D, ShellPath, StepId,
    UserConfigEdit, WIFI_BACKEND_CONFIG,
};
pub use user_config::edit_user_config;

use crate::apply::EventSink;
use crate::report::{failure, note};
use crate::{config_file, repair, secure};

pub const PROC: &str = "proc";
pub const SYSTEM_ROOT: &str = "/";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockRemoval {
    Removed,
    NoLock,
    /// A live transaction owns the lock. Removing it would let a second
    /// one corrupt the database.
    Held {
        pid: u32,
    },
}

/// Runs `builtin` against the system below `root` and the user's home
/// `home`, and reports to `sink`. The helper has no home: edits of user
/// files fail there.
pub fn execute(
    root: &Path,
    home: Option<&Path>,
    step: StepId,
    builtin: &Builtin,
    sink: &mut dyn EventSink,
) -> Outcome {
    match builtin {
        Builtin::RemoveDatabaseLock => database_lock(root, step, sink),
        Builtin::InstallMirrorlist(mirrorlist) => mirrorlist_written(root, step, mirrorlist, sink),
        Builtin::RequireListedShell(shell) => listed_shell(root, step, shell, sink),
        Builtin::WriteDnsConfig(provider) => changed(
            step,
            sink,
            &format!("/{NETWORKMANAGER_CONF_D}/{DNS_CONFIG}"),
            write_dns_config(root, *provider),
        ),
        Builtin::WriteWifiBackend(backend) => changed(
            step,
            sink,
            &format!("/{NETWORKMANAGER_CONF_D}/{WIFI_BACKEND_CONFIG}"),
            write_wifi_backend(root, *backend),
        ),
        Builtin::SetGarudaUpdateFlag { flag, on } => changed(
            step,
            sink,
            &format!("/{GARUDA_UPDATE_CONFIG}"),
            set_garuda_update_flag(root, *flag, *on),
        ),
        Builtin::EditUserConfig { edit, on } => user_config(home, step, edit, *on, sink),
        Builtin::EditLocaleGen { entry, on } => changed(
            step,
            sink,
            &format!("/{LOCALE_GEN}"),
            edit_locale_gen(root, entry, *on),
        ),
        Builtin::SetSystemLocale(locale) => system_locale(root, step, locale, sink),
        Builtin::RequireOrdinaryUser(user) => {
            accounts::require_ordinary_user(root, step, user, sink)
        }
        Builtin::RequireUserGroup(group) => accounts::require_user_group(root, step, group, sink),
        Builtin::RequireFreeHome(folder) => accounts::require_free_home(root, step, folder, sink),
        Builtin::WriteConfigFile(write) => config_file::execute(root, step, write, sink),
        Builtin::WriteHomeFile(write) => home_write(home, step, write, sink),
        Builtin::EditModprobeBlacklist { module, on } => changed(
            step,
            sink,
            &format!("/{MODPROBE_BLACKLIST}"),
            edit_modprobe_blacklist(root, module, *on),
        ),
        Builtin::EditGrubDefaults(settings) => changed(
            step,
            sink,
            &format!("/{GRUB_DEFAULTS}"),
            edit_grub_defaults(root, settings),
        ),
        Builtin::EditLoaderConf(change) => loader_conf(root, step, change, sink),
        Builtin::WriteKernelCmdline(parameters) => kernel_cmdline(root, step, parameters, sink),
        Builtin::PrepareRepairMount => repair::execute_prepare(root, step, sink),
        Builtin::RequireInstalledSystem(need) => repair::execute_require(root, step, *need, sink),
        Builtin::RecheckRepairDevices { system, work } => {
            repair::execute_recheck(root, step, system, work, sink)
        }
        // It runs a program, which only the runner can: `apply` hands it there.
        Builtin::RebuildInitramfs { root: _ } | Builtin::RebuildUkis => {
            failure(step, sink, "a rebuild needs a runner")
        }
    }
}

fn kernel_cmdline(
    root: &Path,
    step: StepId,
    parameters: &toolbox_plan::KernelParameters,
    sink: &mut dyn EventSink,
) -> Outcome {
    changed(
        step,
        sink,
        &format!("/{}", toolbox_plan::sdboot::KERNEL_CMDLINE),
        write_kernel_cmdline(root, parameters),
    )
}

fn loader_conf(
    root: &Path,
    step: StepId,
    change: &toolbox_plan::sdboot::LoaderChange,
    sink: &mut dyn EventSink,
) -> Outcome {
    changed(
        step,
        sink,
        "loader.conf on the EFI system partition",
        loader::edit_loader_conf(root, change),
    )
}

/// # Errors
///
/// When `/proc` cannot be listed or the lock cannot be removed.
pub fn remove_database_lock(proc_root: &Path, lock: &Path) -> io::Result<LockRemoval> {
    let held = match std::fs::metadata(lock) {
        Ok(held) => held,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(LockRemoval::NoLock),
        Err(error) => return Err(error),
    };

    if let Some(pid) = lock_holder(proc_root, &held)? {
        return Ok(LockRemoval::Held { pid });
    }

    match std::fs::remove_file(lock) {
        Ok(()) => Ok(LockRemoval::Removed),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(LockRemoval::NoLock),
        Err(error) => Err(error),
    }
}

/// Writes the Chaotic-AUR mirrorlist below `root` and keeps the old one as
/// `chaotic-mirrorlist.bak`. Returns whether there was an old one.
///
/// # Errors
///
/// When `etc/pacman.d` below `root` is missing or reached through a
/// symlink, or the write fails.
pub fn install_mirrorlist(root: &Path, mirrorlist: &Mirrorlist) -> io::Result<bool> {
    let (dir, name) = secure::open_parent(root, CHAOTIC_MIRRORLIST)?;
    let backup = format!("{name}.bak");

    secure::replace_system_file(&dir, name, mirrorlist.as_str().as_bytes(), Some(&backup))
}

fn rewritten(
    dir: &OwnedFd,
    name: &str,
    current: &str,
    wanted: &str,
    backup: Option<&str>,
) -> io::Result<FileChange> {
    if wanted == current {
        return Ok(FileChange::Unchanged);
    }

    secure::replace_system_file(dir, name, wanted.as_bytes(), backup)?;
    Ok(FileChange::Written)
}

/// libalpm keeps the lock's descriptor until its transaction ends, so
/// this finds pacman, pamac and `PackageKit` alike, whatever their name.
fn lock_holder(proc_root: &Path, lock: &Metadata) -> io::Result<Option<u32>> {
    for entry in std::fs::read_dir(proc_root)? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };

        // A process may exit between listing and reading: no holder then.
        let Ok(open) = std::fs::read_dir(entry.path().join("fd")) else {
            continue;
        };

        let holds = open.flatten().any(|file| {
            std::fs::metadata(file.path())
                .is_ok_and(|file| file.dev() == lock.dev() && file.ino() == lock.ino())
        });

        if holds {
            return Ok(Some(pid));
        }
    }

    Ok(None)
}

fn mirrorlist_written(
    root: &Path,
    step: StepId,
    mirrorlist: &Mirrorlist,
    sink: &mut dyn EventSink,
) -> Outcome {
    match install_mirrorlist(root, mirrorlist) {
        Ok(true) => note(
            step,
            sink,
            "wrote the Chaotic-AUR mirrorlist, the old one is chaotic-mirrorlist.bak",
            Outcome::Succeeded,
        ),
        Ok(false) => note(
            step,
            sink,
            "wrote the Chaotic-AUR mirrorlist",
            Outcome::Succeeded,
        ),
        Err(error) => failure(
            step,
            sink,
            &format!("cannot write the Chaotic-AUR mirrorlist: {error}"),
        ),
    }
}

fn database_lock(root: &Path, step: StepId, sink: &mut dyn EventSink) -> Outcome {
    match remove_database_lock(&root.join(PROC), &root.join(DATABASE_LOCK)) {
        Ok(LockRemoval::Removed) => note(
            step,
            sink,
            "removed the pacman database lock",
            Outcome::Succeeded,
        ),
        Ok(LockRemoval::NoLock) => note(
            step,
            sink,
            "there is no pacman database lock",
            Outcome::Skipped,
        ),
        Ok(LockRemoval::Held { pid }) => failure(
            step,
            sink,
            &format!("process {pid} holds the pacman lock, so it stays"),
        ),
        Err(error) => failure(
            step,
            sink,
            &format!("cannot remove the pacman lock: {error}"),
        ),
    }
}

fn user_config(
    home: Option<&Path>,
    step: StepId,
    edit: &UserConfigEdit,
    on: bool,
    sink: &mut dyn EventSink,
) -> Outcome {
    match home {
        Some(home) => {
            // The display form, with the leading `~/` the user knows.
            let file = edit.path().to_string();

            changed(step, sink, &file, edit_user_config(home, edit, on))
        }
        None => failure(
            step,
            sink,
            &format!("{edit} is a file of the user, so only the app may change it"),
        ),
    }
}

fn home_write(
    home: Option<&Path>,
    step: StepId,
    write: &HomeWrite,
    sink: &mut dyn EventSink,
) -> Outcome {
    let file = format!("~/{}", write.path.as_str());
    match home {
        Some(home) => changed(
            step,
            sink,
            &file,
            write_home_file(home, write).map(|()| FileChange::Written),
        ),
        None => failure(
            step,
            sink,
            &format!("{file} is a file of the user, so only the app may write it"),
        ),
    }
}

fn system_locale(
    root: &Path,
    step: StepId,
    locale: &LocaleName,
    sink: &mut dyn EventSink,
) -> Outcome {
    match set_system_locale(root, locale) {
        Ok(SystemLocaleChange::Changed(change)) => {
            changed(step, sink, &format!("/{LOCALE_CONF}"), Ok(change))
        }
        Ok(SystemLocaleChange::NotEnabled) => failure(
            step,
            sink,
            &format!("{locale} is not enabled in /{LOCALE_GEN}, so it cannot be the system locale"),
        ),
        Err(error) => failure(
            step,
            sink,
            &format!("cannot change /{LOCALE_CONF}: {error}"),
        ),
    }
}

/// A file that was already as wanted skips, so the Console shows that
/// nothing changed.
fn changed(
    step: StepId,
    sink: &mut dyn EventSink,
    file: &str,
    result: io::Result<FileChange>,
) -> Outcome {
    match result {
        Ok(FileChange::Written) => note(step, sink, &format!("wrote {file}"), Outcome::Succeeded),
        Ok(FileChange::Removed) => note(step, sink, &format!("removed {file}"), Outcome::Succeeded),
        Ok(FileChange::Unchanged) => note(
            step,
            sink,
            &format!("{file} is already as wanted"),
            Outcome::Skipped,
        ),
        Err(error) => failure(step, sink, &format!("cannot change {file}: {error}")),
    }
}

fn listed_shell(root: &Path, step: StepId, shell: &ShellPath, sink: &mut dyn EventSink) -> Outcome {
    match shell_is_listed(root, shell) {
        Ok(true) => note(
            step,
            sink,
            &format!("{shell} is listed in /{SHELLS}"),
            Outcome::Succeeded,
        ),
        Ok(false) => failure(
            step,
            sink,
            &format!("{shell} is not listed in /{SHELLS}, so it cannot be a login shell"),
        ),
        Err(error) => failure(step, sink, &format!("cannot read /{SHELLS}: {error}")),
    }
}
