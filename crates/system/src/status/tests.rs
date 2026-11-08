use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use toolbox_plan::{PackageName, UserName};

use super::{
    AurUpdateSource, HealthIssue, HealthLevel, HealthSource, MergeSource, PendingUpdate, Status,
    StatusSources, UpdateLogSource, UpdateSource, probe_status,
};
use crate::{HostSource, ProbeError};

struct FakeUpdates(Result<Vec<PendingUpdate>, ProbeError>);

struct FakeAur(Result<Vec<PendingUpdate>, ProbeError>);

struct FakeMerges(Result<Vec<PathBuf>, ProbeError>);

struct FakeLog(Result<Option<SystemTime>, ProbeError>);

/// Remembers being asked, so a skipped probe shows.
struct FakeHealth {
    issues: Result<Vec<HealthIssue>, ProbeError>,
    asked: AtomicBool,
}

struct FakeHost(Result<bool, ProbeError>);

struct PanickingUpdates;

impl UpdateSource for FakeUpdates {
    fn pending(&self) -> Result<Vec<PendingUpdate>, ProbeError> {
        self.0.clone()
    }
}

impl AurUpdateSource for FakeAur {
    fn pending_aur(&self) -> Result<Vec<PendingUpdate>, ProbeError> {
        self.0.clone()
    }
}

impl MergeSource for FakeMerges {
    fn waiting(&self) -> Result<Vec<PathBuf>, ProbeError> {
        self.0.clone()
    }
}

impl UpdateLogSource for FakeLog {
    fn last_update(&self) -> Result<Option<SystemTime>, ProbeError> {
        self.0.clone()
    }
}

impl HealthSource for FakeHealth {
    fn issues(&self) -> Result<Vec<HealthIssue>, ProbeError> {
        self.asked.store(true, Ordering::SeqCst);
        self.issues.clone()
    }
}

impl HostSource for FakeHost {
    fn user(&self) -> Result<UserName, ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn is_live_iso(&self) -> Result<bool, ProbeError> {
        self.0.clone()
    }

    fn desktops(&self) -> Result<Vec<String>, ProbeError> {
        Ok(Vec::new())
    }

    fn edition(&self) -> Result<Option<String>, ProbeError> {
        Ok(None)
    }

    fn is_reboot_pending(&self) -> Result<bool, ProbeError> {
        Ok(false)
    }
}

impl UpdateSource for PanickingUpdates {
    fn pending(&self) -> Result<Vec<PendingUpdate>, ProbeError> {
        panic!("boom");
    }
}

fn at(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

fn update(name: &str) -> PendingUpdate {
    PendingUpdate {
        name: PackageName::new(name).unwrap(),
        from: "1-1".to_owned(),
        to: "2-1".to_owned(),
    }
}

fn warning() -> HealthIssue {
    HealthIssue {
        level: HealthLevel::Low,
        message: "careful".to_owned(),
        fix_available: false,
    }
}

fn sources(live_iso: Result<bool, ProbeError>) -> (StatusSources, Arc<FakeHealth>) {
    let health = Arc::new(FakeHealth {
        issues: Ok(vec![warning()]),
        asked: AtomicBool::new(false),
    });

    let sources = StatusSources {
        updates: Arc::new(FakeUpdates(Ok(vec![update("bash")]))),
        aur_updates: Arc::new(FakeAur(Err(ProbeError::Unavailable))),
        merges: Arc::new(FakeMerges(Ok(vec![PathBuf::from("/etc/a.pacnew")]))),
        update_log: Arc::new(FakeLog(Ok(Some(at(1_000))))),
        health: Arc::<FakeHealth>::clone(&health),
        host: Arc::new(FakeHost(live_iso)),
    };

    (sources, health)
}

#[test]
fn every_probe_lands_in_its_own_field() {
    let (sources, health) = sources(Ok(false));

    assert_eq!(
        probe_status(&sources),
        Status {
            updates: Ok(vec![update("bash")]),
            aur_updates: Err(ProbeError::Unavailable),
            merges: Ok(vec![PathBuf::from("/etc/a.pacnew")]),
            last_update: Ok(Some(at(1_000))),
            health: Ok(vec![warning()]),
        }
    );
    assert!(health.asked.load(Ordering::SeqCst));
}

#[test]
fn failures_stay_with_their_field() {
    let (mut sources, _health) = sources(Ok(false));
    sources.updates = Arc::new(FakeUpdates(Err(ProbeError::Failed("offline".to_owned()))));
    sources.update_log = Arc::new(FakeLog(Err(ProbeError::Unavailable)));

    let status = probe_status(&sources);

    assert_eq!(
        status.updates,
        Err(ProbeError::Failed("offline".to_owned()))
    );
    assert_eq!(status.last_update, Err(ProbeError::Unavailable));
    assert_eq!(status.merges, Ok(vec![PathBuf::from("/etc/a.pacnew")]));
}

/// garuda-health checks an installed system, the ISO has nothing to fix.
#[test]
fn health_is_skipped_on_the_live_iso() {
    let (sources, health) = sources(Ok(true));

    assert_eq!(probe_status(&sources).health, Err(ProbeError::Unavailable));
    assert!(!health.asked.load(Ordering::SeqCst));
}

#[test]
fn health_runs_when_the_live_check_fails() {
    let (sources, health) = sources(Err(ProbeError::Unavailable));

    assert_eq!(probe_status(&sources).health, Ok(vec![warning()]));
    assert!(health.asked.load(Ordering::SeqCst));
}

#[test]
fn a_panicking_probe_fails_only_its_field() {
    let (mut sources, _health) = sources(Ok(false));
    sources.updates = Arc::new(PanickingUpdates);

    let status = probe_status(&sources);

    assert_eq!(
        status.updates,
        Err(ProbeError::Failed("the probe panicked".to_owned()))
    );
    assert_eq!(status.aur_updates, Err(ProbeError::Unavailable));
}
