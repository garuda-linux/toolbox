//! What the helper does itself for a boot repair: it looks at the devices
//! before anything runs, makes the folder the installed system is mounted
//! at, and checks that what was mounted is a system GRUB can be installed
//! into. Mounting, unlocking and GRUB's programs are commands of the
//! procedure in `toolbox-plan`.

mod disks;
mod preflight;
#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use preflight::Moment;
pub use preflight::PreflightError;
use rustix::fs::Mode;
use rustix::io::Errno;
use toolbox_plan::procedure::Program;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    ESP_FOLDER, InstalledSystem, REPAIR_ROOT, RepairMount, RepairNeed, RepairWork, RootAction,
    StepId,
};

use crate::apply::EventSink;
use crate::builtin::SYSTEM_ROOT;
use crate::report::{failure, note};
use crate::secure;

/// The kernels Arch packages install into `/boot`.
const KERNEL_PREFIX: &str = "vmlinuz";

/// What the mounted file system lacks to be repaired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lacking {
    /// No `/usr/bin` with the program: not an installed system with
    /// GRUB, or a btrfs whose system lies in another subvolume.
    Program(Program),
    /// No kernel in `/boot`: the system keeps it on a partition of its
    /// own, which this repair does not mount.
    Kernel,
    /// No `/boot/efi`: the system mounts its EFI partition elsewhere.
    EspFolder,
    /// A folder the repair binds onto is missing or a symlink.
    /// `mount --bind` follows a link at its target, so the live system's
    /// `/dev` would land wherever the installed system points it.
    BindTarget(RepairMount),
}

/// Checks the devices of every boot repair among `actions` against the
/// running system. The helper calls it before the first step runs.
///
/// # Errors
///
/// The reason the first unusable device is refused, for the Console.
pub fn preflight<'actions>(
    actions: impl IntoIterator<Item = &'actions RootAction>,
) -> Result<(), PreflightError> {
    let system = preflight::System {
        root: Path::new(SYSTEM_ROOT),
        nodes: &disks::DeviceNodes,
    };

    actions
        .into_iter()
        .try_for_each(|action| preflight::check(&system, action))
}

/// Makes the mount point below `root`, in a folder only root may enter.
///
/// # Errors
///
/// When a folder on the way is a symlink or cannot be made, the mount
/// table cannot be read, or something is mounted at the mount point.
pub fn prepare_mount(root: &Path) -> io::Result<()> {
    let target = Path::new(REPAIR_ROOT);
    let folder = target
        .parent()
        .ok_or_else(|| io::Error::other("the mount point has no folder"))?;

    let private = secure::create_dirs(root, folder)?;

    rustix::fs::fchmod(&private, Mode::RWXU)?;
    secure::create_dirs(root, target)?;

    match disks::mounts(root)?
        .into_iter()
        .find(|mount| below_repair_root(&mount.point))
    {
        Some(mount) => Err(io::Error::other(PreflightError::Leftover {
            point: mount.point,
        })),
        None => Ok(()),
    }
}

/// Looks into the file system mounted at the repair's mount point below
/// `root`, without following a symlink on the way.
///
/// # Errors
///
/// When a folder cannot be read for another reason than being absent.
pub fn installed_system(root: &Path, need: RepairNeed) -> io::Result<Option<Lacking>> {
    let system = Path::new(REPAIR_ROOT);
    let programs: &[Program] = match need {
        RepairNeed::Menu => &[Program::GrubMkconfig],
        RepairNeed::Installer | RepairNeed::InstallerWithEsp => {
            &[Program::GrubInstall, Program::GrubMkconfig]
        }
    };

    let bin = absent_as_none(secure::open_dir(root, &system.join("usr/bin")))?;

    for program in programs {
        let present = bin
            .as_ref()
            .is_some_and(|bin| secure::is_regular(bin, program.name()));

        if !present {
            return Ok(Some(Lacking::Program(*program)));
        }
    }

    let kernels = match absent_as_none(secure::open_dir(root, &system.join("boot")))? {
        Some(boot) => secure::regular_files(&boot)?,
        None => Vec::new(),
    };

    if !kernels.iter().any(|name| name.starts_with(KERNEL_PREFIX)) {
        return Ok(Some(Lacking::Kernel));
    }

    if need == RepairNeed::InstallerWithEsp
        && absent_as_none(secure::open_dir(root, &system.join(ESP_FOLDER)))?.is_none()
    {
        return Ok(Some(Lacking::EspFolder));
    }

    for place in RepairMount::BINDS {
        if !is_folder(root, &system.join(place.inside()))? {
            return Ok(Some(Lacking::BindTarget(place)));
        }
    }

    Ok(None)
}

pub fn execute_prepare(root: &Path, step: StepId, sink: &mut dyn EventSink) -> Outcome {
    match prepare_mount(root) {
        Ok(()) => note(
            step,
            sink,
            &format!("/{REPAIR_ROOT} is ready"),
            Outcome::Succeeded,
        ),
        Err(error) => failure(
            step,
            sink,
            &format!("cannot prepare /{REPAIR_ROOT}: {error}"),
        ),
    }
}

pub fn execute_require(
    root: &Path,
    step: StepId,
    need: RepairNeed,
    sink: &mut dyn EventSink,
) -> Outcome {
    let message = match installed_system(root, need) {
        Ok(None) => {
            return note(
                step,
                sink,
                "the partition holds an installed system with GRUB",
                Outcome::Succeeded,
            );
        }
        Ok(Some(Lacking::Program(program))) => format!(
            "the partition has no {}: it is no installed system with GRUB, or its system lies in another subvolume",
            program.path()
        ),
        Ok(Some(Lacking::Kernel)) => {
            "the system has no kernel in /boot: a /boot on a partition of its own is not supported"
                .to_owned()
        }
        Ok(Some(Lacking::EspFolder)) => format!(
            "the system has no /{ESP_FOLDER}: it mounts its EFI system partition elsewhere, which is not supported"
        ),
        Ok(Some(Lacking::BindTarget(place))) => format!(
            "the system's /{} is missing or a symlink, so nothing can be mounted there",
            place.inside()
        ),
        Err(error) => format!("cannot look into the mounted system: {error}"),
    };

    failure(step, sink, &message)
}

pub fn execute_recheck(
    root: &Path,
    step: StepId,
    system: &InstalledSystem,
    work: &RepairWork,
    sink: &mut dyn EventSink,
) -> Outcome {
    let seen = preflight::System {
        root,
        nodes: &disks::DeviceNodes,
    };

    match preflight::devices(&seen, system, work, Moment::During) {
        Ok(()) => note(
            step,
            sink,
            "the devices of the repair are still usable",
            Outcome::Succeeded,
        ),
        Err(reason) => failure(step, sink, &reason.to_string()),
    }
}

fn below_repair_root(point: &str) -> bool {
    point
        .strip_prefix('/')
        .and_then(|rest| rest.strip_prefix(REPAIR_ROOT))
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// A folder that is not there is an answer, not a failure.
fn absent_as_none<T>(result: io::Result<T>) -> io::Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Whether `folder` below `root` is a real folder. An open that follows
/// no link answers `ELOOP` or `ENOTDIR` for a symlink.
fn is_folder(root: &Path, folder: &Path) -> io::Result<bool> {
    match secure::open_dir(root, folder) {
        Ok(_) => Ok(true),
        Err(error) => match Errno::from_io_error(&error) {
            Some(Errno::NOENT | Errno::LOOP | Errno::NOTDIR) => Ok(false),
            Some(_) | None => Err(error),
        },
    }
}
