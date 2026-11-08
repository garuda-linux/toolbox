//! Facts about the machine and the session the app runs in.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use toolbox_plan::UserName;

use crate::error::ProbeError;
use crate::{accounts, files};

/// Filesystems an ISO mounts as its root, the writable layer over the
/// read-only image.
const LIVE_FILESYSTEMS: [&str; 2] = ["overlay", "aufs"];

const CONTAINER_MARKERS: [&str; 2] = ["run/.containerenv", ".dockerenv"];

/// The environment of the running process, for [`SystemHost::from_process`].
pub type ProcessEnv = fn(&str) -> Option<OsString>;

/// The host facts as one snapshot part. A fact whose probe failed keeps its
/// default here, the probe issue says why.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Host {
    /// The account the app runs for.
    pub user: Option<UserName>,
    /// Running from the installation medium, not an installed system.
    pub live_iso: bool,
    /// `XDG_CURRENT_DESKTOP`, most specific first.
    pub desktops: Vec<String>,
    /// The Garuda edition in lower case (`dr460nized`, `gnome`, …). `None`
    /// on other Arch based systems.
    pub edition: Option<String>,
    /// A system update finished after the last boot.
    pub reboot_pending: bool,
}

pub trait HostSource: Send + Sync {
    /// The user the app runs as: its effective user id, named by
    /// `/etc/passwd`. `USER` survives `sudo -u` and `su` without `-l`, so
    /// the guards would protect the wrong account.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] when the user id is missing from
    /// `passwd` (an account from LDAP or sssd), [`ProbeError::Failed`]
    /// when `/proc/self/status` names no user id, or a file read error.
    fn user(&self) -> Result<UserName, ProbeError>;

    /// Whether `/` is the live ISO's overlay. A container runs on an overlay too, so
    /// inside one this is never the live ISO.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a mount table,
    /// [`ProbeError::Failed`] when it lists no root.
    fn is_live_iso(&self) -> Result<bool, ProbeError>;

    /// The current desktops, empty outside a desktop session.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Failed`] when the source cannot be read.
    fn desktops(&self) -> Result<Vec<String>, ProbeError>;

    /// The Garuda edition, `None` when the system does not name one.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Failed`] when the release file cannot be read.
    fn edition(&self) -> Result<Option<String>, ProbeError>;

    /// Whether a system update finished after the last boot.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] when the boot time is unknown,
    /// [`ProbeError::Failed`] when a source cannot be read.
    fn is_reboot_pending(&self) -> Result<bool, ProbeError>;
}

/// Where [`SystemHost`] reads from. Tests point these into a temporary
/// folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostRoots {
    /// `/`, for the marker files containers leave.
    pub root: PathBuf,
    /// `/proc`, for the mount table, the boot time and the user id.
    pub proc: PathBuf,
    /// `/usr/lib/garuda`, for `garuda-release`.
    pub garuda_lib: PathBuf,
    /// `/var/lib/garuda`, for the `last_update` stamp.
    pub garuda_state: PathBuf,
    /// `/etc`, for the name of the user the app runs as.
    pub etc: PathBuf,
}

#[derive(Clone, Debug)]
pub struct SystemHost<F> {
    roots: HostRoots,
    lookup: F,
}

impl HostRoots {
    #[must_use = "the paths are only computed, nothing is read"]
    pub fn system() -> Self {
        Self {
            root: PathBuf::from("/"),
            proc: PathBuf::from("/proc"),
            garuda_lib: PathBuf::from("/usr/lib/garuda"),
            garuda_state: PathBuf::from("/var/lib/garuda"),
            etc: PathBuf::from("/etc"),
        }
    }
}

impl SystemHost<ProcessEnv> {
    #[must_use = "the adapter does nothing until asked"]
    pub fn from_process() -> Self {
        Self::new(HostRoots::system(), |key| std::env::var_os(key))
    }
}

impl<F> SystemHost<F>
where
    F: Fn(&str) -> Option<OsString>,
{
    /// `lookup` answers environment variables, so tests never depend on
    /// the real ones.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) const fn new(roots: HostRoots, lookup: F) -> Self {
        Self { roots, lookup }
    }

    /// The marks `systemd-detect-virt --container` reads that need no root:
    /// podman and systemd-nspawn set `container`, podman and Docker leave
    /// a file.
    fn in_container(&self) -> Result<bool, ProbeError> {
        if self
            .variable("container")?
            .is_some_and(|name| !name.is_empty())
        {
            return Ok(true);
        }

        Ok(CONTAINER_MARKERS
            .iter()
            .any(|marker| self.roots.root.join(marker).exists()))
    }

    fn variable(&self, key: &str) -> Result<Option<String>, ProbeError> {
        (self.lookup)(key)
            .map(|value| {
                value.into_string().map_err(|raw| {
                    ProbeError::Failed(format!("{key} is not UTF-8: {}", raw.display()))
                })
            })
            .transpose()
    }
}

impl<F> HostSource for SystemHost<F>
where
    F: Fn(&str) -> Option<OsString> + Send + Sync,
{
    fn user(&self) -> Result<UserName, ProbeError> {
        let path = self.roots.proc.join("self/status");
        let uid = effective_uid(&files::read(&path)?)
            .ok_or_else(|| ProbeError::Failed(format!("{} names no user id", path.display())))?;

        let passwd = files::read(&self.roots.etc.join("passwd"))?;

        accounts::name_of(&passwd, uid).ok_or(ProbeError::Unavailable)
    }

    fn is_live_iso(&self) -> Result<bool, ProbeError> {
        let path = self.roots.proc.join("self/mounts");
        let mounts = files::read(&path)?;

        let live = root_is_live(&mounts)
            .ok_or_else(|| ProbeError::Failed(format!("{} lists no root", path.display())))?;

        Ok(live && !self.in_container()?)
    }

    fn desktops(&self) -> Result<Vec<String>, ProbeError> {
        Ok(self
            .variable("XDG_CURRENT_DESKTOP")?
            .map(|value| parse_desktops(&value))
            .unwrap_or_default())
    }

    fn edition(&self) -> Result<Option<String>, ProbeError> {
        let release = files::read_optional(&self.roots.garuda_lib.join("garuda-release"))?;

        Ok(release.as_deref().and_then(parse_edition))
    }

    /// Updates write `last_update` when they finish. Without it no update
    /// ran through Garuda's tools, so nothing waits for a reboot.
    fn is_reboot_pending(&self) -> Result<bool, ProbeError> {
        let stamp = self.roots.garuda_state.join("last_update");
        let Some(updated) = files::modified_optional(&stamp)? else {
            return Ok(false);
        };

        let stat = files::read(&self.roots.proc.join("stat"))?;
        let booted = boot_time(&stat)?;

        Ok(updated > booted)
    }
}

/// The second field of `Uid:` in `/proc/<pid>/status`: real, effective,
/// saved and file system user id follow each other.
fn effective_uid(status: &str) -> Option<u32> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// Whether the root filesystem in a `/proc/self/mounts` table is a live
/// overlay. A later mount on `/` hides the earlier ones, so the last wins.
fn root_is_live(mounts: &str) -> Option<bool> {
    mounts
        .lines()
        .rev()
        .find_map(|line| {
            let mut fields = line.split_whitespace();
            let mount_point = fields.nth(1)?;
            let filesystem = fields.next()?;

            (mount_point == "/").then_some(filesystem)
        })
        .map(|filesystem| LIVE_FILESYSTEMS.contains(&filesystem))
}

/// `EDITION=` from `garuda-release`. Releases write it quoted or not and
/// in any case, so it is unquoted and lowered for comparison.
fn parse_edition(release: &str) -> Option<String> {
    release.lines().find_map(|line| {
        let value = line.trim().strip_prefix("EDITION=")?;
        let value = value.trim().trim_matches(['"', '\'']).to_lowercase();

        (!value.is_empty()).then_some(value)
    })
}

/// Seconds since the epoch at boot, from the `btime` line of `/proc/stat`.
fn parse_boot_time(stat: &str) -> Option<u64> {
    stat.lines()
        .find_map(|line| line.strip_prefix("btime ")?.trim().parse().ok())
}

pub fn boot_time(stat: &str) -> Result<SystemTime, ProbeError> {
    parse_boot_time(stat)
        .and_then(|seconds| SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds)))
        .ok_or_else(|| ProbeError::Failed("no boot time in /proc/stat".to_owned()))
}

fn parse_desktops(value: &str) -> Vec<String> {
    value
        .split(':')
        .filter(|desktop| !desktop.is_empty())
        .map(str::to_owned)
        .collect()
}
