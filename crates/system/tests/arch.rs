//! The status and package probes against the Arch test image
//! (`ci/arch/Containerfile`): real checkupdates, pacdiff, pacman.log and
//! libalpm databases. They only run there:
//! `TOOLBOX_IT_ARCH=1`, root and `/etc/arch-release`. Elsewhere they
//! return at once. Run them with
//! `scripts/arch-it.sh -p toolbox-system --test arch`.
//!
//! The image has `toolbox-test-up` 1.0 installed and its local repository
//! offers 1.1. pacman-contrib and fakeroot (base-devel) are installed, paru
//! and garuda-health are not.

mod common;

use std::error::Error;
use std::fs;
use std::io::Write as _;
use std::process::Command;
use std::time::{Duration, SystemTime};

use toolbox_plan::PackageName;
use toolbox_system::{
    AppIconSource as _, AppStreamIcons, AurUpdateSource as _, Checkupdates, Dkms, DkmsSource as _,
    EtcLocales, KernelSource as _, Libalpm, LinuxPackageSource as _, LocaleSource as _,
    MergeSource as _, Pacdiff, PackageSource as _, PacmanLog, Paru, PendingUpdate, ProbeError,
    StatusSources, SyncSource as _, SystemKernels, UpdateLogSource as _, UpdateSource as _,
    probe_status,
};

const PACMAN_LOG: &str = "/var/log/pacman.log";

type TestResult = Result<(), Box<dyn Error>>;

/// Output goes to stderr, which `cargo test` shows when the test fails.
fn show(program: &str, args: &[&str]) -> TestResult {
    let mut stderr = std::io::stderr();

    match Command::new(program).args(args).output() {
        Ok(output) => writeln!(
            stderr,
            "{program} {}: {}\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )?,
        Err(error) => writeln!(stderr, "{program}: {error}")?,
    }

    Ok(())
}

fn show_log_tail() -> TestResult {
    let log = fs::read(PACMAN_LOG)?;
    let text = String::from_utf8_lossy(&log);
    let lines: Vec<&str> = text.lines().collect();
    let tail = lines
        .get(lines.len().saturating_sub(15)..)
        .unwrap_or_default();

    writeln!(
        std::io::stderr(),
        "tail of {PACMAN_LOG}:\n{}",
        tail.join("\n")
    )?;
    Ok(())
}

/// checkupdates syncs a copy of the test repository's database into
/// `/tmp/checkup-db-0` under fakeroot. As root that works the same: the
/// script (pacman-contrib 1.13.1) has no root check.
#[test]
fn checkupdates_reports_the_test_package() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let result = Checkupdates::new().pending();
    let expected = PendingUpdate {
        name: PackageName::new("toolbox-test-up")?,
        from: "1.0-1".to_owned(),
        to: "1.1-1".to_owned(),
    };

    if result != Ok(vec![expected.clone()]) {
        show("checkupdates", &["--nocolor"])?;
        show("pacman", &["-Q", "toolbox-test-up"])?;
    }

    assert_eq!(result, Ok(vec![expected]));
    Ok(())
}

/// The image build installed `toolbox-test-up` with pacman, so the log
/// ends with a completed transaction. Podman may reuse an old image layer,
/// so the test compares with the log's own modification time, not with
/// today. Only the hooks of that transaction write to the log after it.
#[test]
fn pacman_log_has_a_completed_transaction() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let now = SystemTime::now();
    let result = PacmanLog::system().last_update();
    let written = fs::metadata(PACMAN_LOG)?.modified()?;

    let Ok(Some(last)) = result else {
        show_log_tail()?;
        return Err(format!("no completed transaction in {PACMAN_LOG}").into());
    };

    let close_to_last_write = written
        .duration_since(last)
        .is_ok_and(|gap| gap < Duration::from_secs(3_600));

    if !close_to_last_write || last > now {
        show_log_tail()?;
    }

    assert!(
        close_to_last_write,
        "the last transaction should be within an hour before the log's last write"
    );
    assert!(last <= now, "the last transaction lies in the future");

    // The row building for the Kernels page lives in toolbox-core (which
    // this crate must not link). The age note only tells whose image is
    // old, so it counts fourteen days here itself.
    if now
        .duration_since(last)
        .is_ok_and(|age| age > Duration::from_hours(14 * 24))
    {
        writeln!(
            std::io::stderr(),
            "note: the image is older than two weeks, rebuild it for fresh logs"
        )?;
    }

    Ok(())
}

/// The image has no merges waiting, but pacdiff must run and answer.
#[test]
fn pacdiff_lists_waiting_files() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let result = Pacdiff::new().waiting();

    if result.is_err() {
        show("pacdiff", &["-o"])?;
    }

    let files = result?;

    assert!(
        files.iter().all(|file| file.is_absolute() && file.exists()),
        "pacdiff listed a path that does not exist"
    );
    Ok(())
}

/// libalpm reads the image's local database with the test package the
/// image build installed. Other tests may install more, so only that one
/// is checked.
#[test]
fn libalpm_lists_the_installed_test_package() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let installed = Libalpm::system().installed()?;

    if !installed.0.contains(&PackageName::new("toolbox-test-up")?) {
        show("pacman", &["-Qq"])?;
    }

    assert!(installed.0.contains(&PackageName::new("toolbox-test-up")?));
    Ok(())
}

/// The image's `pacman.conf` knows only the test repository, synced by
/// `setup.sh`, so libalpm offers exactly its packages.
#[test]
fn libalpm_offers_the_test_repository() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let offered = Libalpm::system().offered()?;
    let expected: Vec<PackageName> = [
        "linux-toolbox-test",
        "linux-toolbox-test-headers",
        "toolbox-test-a",
        "toolbox-test-conflict",
        "toolbox-test-dep",
        "toolbox-test-up",
    ]
    .into_iter()
    .map(PackageName::new)
    .collect::<Result<_, _>>()?;

    if offered.0.iter().ne(expected.iter()) {
        show("pacman", &["-Sl"])?;
    }

    assert_eq!(offered.0.into_iter().collect::<Vec<_>>(), expected);
    Ok(())
}

/// paru is not in the image. As root it would be refused anyway.
#[test]
fn paru_is_unavailable() {
    if !common::in_test_image() {
        return;
    }

    assert_eq!(Paru::new().pending_aur(), Err(ProbeError::Unavailable));
}

/// All probes together, as the app runs them. garuda-health is not in
/// the image (and the container root is an overlay, which reads as the
/// live ISO), so health is unavailable.
#[test]
fn probe_status_fills_every_card() {
    if !common::in_test_image() {
        return;
    }

    let status = probe_status(&StatusSources::system());

    assert_eq!(status.aur_updates, Err(ProbeError::Unavailable));
    assert_eq!(status.health, Err(ProbeError::Unavailable));

    for (card, failure) in [
        ("updates", status.updates.as_ref().err()),
        ("merges", status.merges.as_ref().err()),
        ("last update", status.last_update.as_ref().err()),
    ] {
        assert_eq!(failure, None, "{card} failed");
    }
}

/// Arch's own icon cache (`archlinux-appstream-data`) is in the image, so
/// the folder layout and file naming the lookup assumes are checked against
/// the real thing.
#[test]
fn appstream_icons_are_found_for_well_known_apps() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let icons = AppStreamIcons::system();

    if icons.is_empty() {
        show("find", &["/usr/share/swcatalog", "-maxdepth", "3"])?;
        return Err("no AppStream icon found below /usr/share/swcatalog".into());
    }

    for package in ["firefox", "vlc", "gimp"] {
        let Some(icon) = icons.icon_file(package) else {
            show(
                "sh",
                &["-c", "ls /usr/share/swcatalog/icons/*/64x64 | head -40"],
            )?;
            return Err(format!("no icon for {package}").into());
        };

        assert!(icon.is_file(), "{} is listed but missing", icon.display());
    }

    Ok(())
}

/// The test repository offers `linux-toolbox-test` with its headers.
/// Pairing them is toolbox-core logic, which this crate must not link, so
/// the test checks what libalpm delivers for the pair itself.
#[test]
fn libalpm_lists_the_test_kernel_with_its_repository() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let packages = Libalpm::system().linux_packages()?;
    let offered = |name: &str| {
        packages
            .sync
            .iter()
            .find(|package| package.name.as_str() == name)
            .ok_or_else(|| format!("{name} is not offered"))
    };

    let kernel = offered("linux-toolbox-test")?;
    let headers = offered("linux-toolbox-test-headers")?;
    let description = "Garuda Toolbox integration test package";

    assert_eq!(kernel.repo, "toolbox-test");
    assert_eq!(kernel.version, "1.0-1");
    assert_eq!(kernel.description, description);
    assert_eq!(headers.repo, "toolbox-test");
    assert_eq!(headers.version, "1.0-1");
    assert!(
        packages
            .local
            .iter()
            .all(|package| { package.name != kernel.name && package.name != headers.name }),
        "the test kernel is not installed"
    );
    Ok(())
}

/// The container shares the host's kernel, whose modules are not in the
/// image: the release is known, its package is not.
#[test]
fn the_running_kernel_has_a_release() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let running = SystemKernels::system().running()?;

    assert_ne!(running.release.as_str(), "");
    assert_eq!(
        running.release.as_str(),
        fs::read_to_string("/proc/sys/kernel/osrelease")?.trim()
    );
    Ok(())
}

#[test]
fn dkms_is_unavailable() {
    if !common::in_test_image() {
        return;
    }

    assert_eq!(Dkms::system().status(), Err(ProbeError::Unavailable));
}

/// glibc's own list, which the official image drops and
/// `ci/arch/install-deps.sh` brings back: the Locales page reads it.
#[test]
fn the_supported_locales_are_read_from_glibc() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let supported = EtcLocales::system()
        .supported()
        .map_err(|error| format!("/usr/share/i18n/SUPPORTED: {error}"))?;

    let names: Vec<String> = supported
        .0
        .iter()
        .map(|entry| entry.locale.to_string())
        .collect();

    assert!(names.len() > 100, "only {} locales", names.len());
    assert!(names.iter().any(|name| name == "de_DE.UTF-8"));
    assert!(names.iter().any(|name| name == "sr_RS@latin"));
    Ok(())
}

/// The sources glibc builds its locales from name each locale's language
/// and territory: the Locales page shows and searches them.
#[test]
fn the_locales_are_named_by_the_sources_of_glibc() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let names = EtcLocales::system().names();
    let german = names
        .0
        .get("de_DE")
        .ok_or("/usr/share/i18n/locales/de_DE names no language")?;

    assert_eq!(
        (german.language.as_str(), german.territory.as_str()),
        ("German", "Germany")
    );
    assert!(names.0.len() > 100, "only {} names", names.0.len());
    assert!(names.0.contains_key("sr_RS@latin"));
    assert!(
        names
            .0
            .values()
            .all(|name| !name.language.contains("<U") && !name.language.contains('"')),
        "a name was not decoded"
    );
    Ok(())
}
