use toolbox_cli::Exit;
use toolbox_cli::args::{Confirm, ModuleArgs, Read};
use toolbox_core::Snapshot;
use toolbox_core::features::network::{DefaultRoute, Device, Radio, RadioAccess, RadioTarget};
use toolbox_plan::HostName;
use toolbox_system::{NetworkSource, ProbeError};

use super::{address_shown, blacklist, ping, status, traceroute, view};
use crate::out::{Out, Settings};

struct Broken;

impl NetworkSource for Broken {
    fn default_route(&self) -> Result<DefaultRoute, ProbeError> {
        Err(ProbeError::Failed("no route table".to_owned()))
    }

    fn radios(&self) -> Result<Vec<Radio>, ProbeError> {
        Err(ProbeError::Failed("rfkill refused".to_owned()))
    }

    fn radio_access(&self) -> RadioAccess {
        RadioAccess::Missing
    }

    fn set_radio(&self, _target: RadioTarget, _blocked: bool) -> Result<(), ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn pci_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn usb_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }
}

struct NoRoute;

impl NetworkSource for NoRoute {
    fn default_route(&self) -> Result<DefaultRoute, ProbeError> {
        Err(ProbeError::Failed("no route table".to_owned()))
    }

    fn radios(&self) -> Result<Vec<Radio>, ProbeError> {
        Ok(Vec::new())
    }

    fn radio_access(&self) -> RadioAccess {
        RadioAccess::Missing
    }

    fn set_radio(&self, _target: RadioTarget, _blocked: bool) -> Result<(), ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn pci_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn usb_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }
}

fn host() -> HostName {
    HostName::new("garudalinux.org").unwrap()
}

#[test]
fn a_ping_takes_the_page_s_bounds() {
    ping(&host(), 4, 5).unwrap();
    ping(&host(), 0, 5).unwrap_err();
    ping(&host(), 4, 600).unwrap_err();
}

#[test]
fn a_trace_is_one_step() {
    let plan = traceroute(&host(), 30).unwrap();
    assert_eq!(plan.steps().len(), 1);
}

#[test]
fn without_the_drivers_read_nothing_is_blacklisted() {
    let args = ModuleArgs {
        module: "r8169".parse().unwrap(),
        confirm: Confirm::default(),
    };

    let refused = blacklist(&args, true, &Snapshot::default()).unwrap_err();
    assert!(refused.message().contains("drivers could not be read"));
}

#[test]
fn a_failed_read_is_no_missing_router_and_ends_as_a_failure() {
    assert_eq!(view(&Broken).unread, ["router", "radios"]);

    let mut out = Vec::new();
    let exit = status(
        Read { json: false },
        &Broken,
        &mut Out::new(&mut out, Settings::default()),
    );

    let text = String::from_utf8(out).unwrap();

    assert_eq!(exit, Exit::Failure);
    assert!(text.contains("router     not read\n"), "{text}");
    assert!(!text.contains("error:"), "{text}");
}

#[test]
fn drivers_are_on_unless_blacklisted() {
    use toolbox_cli::output::Mark;

    use super::{DriverLine, driver_listing};

    let line = |module: &str, loaded, blacklisted| DriverLine {
        module: module.to_owned(),
        category: "ethernet",
        loaded,
        blacklisted,
    };

    let rows = [line("r8169", true, false), line("e1000", false, true)];
    let listing = driver_listing(&rows);
    let marks: Vec<Mark> = listing.rows.iter().map(|shown| shown.mark).collect();

    assert_eq!(marks, [Mark::On, Mark::Off]);
    assert_eq!(
        listing.summary.as_deref(),
        Some("2, 1 loaded, 1 blacklisted")
    );
    assert!(listing.hint.unwrap().contains("network drivers blacklist"));
}

#[test]
fn the_radios_are_a_marked_list() {
    use toolbox_cli::output::Mark;

    use super::{StatusView, radio_listing};

    let view = StatusView {
        wifi: Some(true),
        bluetooth: Some(false),
        ..StatusView::default()
    };

    let listing = radio_listing(&view);
    let marks: Vec<Mark> = listing.rows.iter().map(|shown| shown.mark).collect();

    assert_eq!(marks, [Mark::On, Mark::Off, Mark::Off]);
    assert_eq!(listing.summary.as_deref(), Some("1 on"));
}

#[test]
fn a_failed_read_keeps_the_program_s_words_out_of_the_output() {
    let mut out = Vec::new();
    status(
        Read { json: false },
        &Broken,
        &mut Out::new(&mut out, Settings::default()),
    );
    let text = String::from_utf8(out).unwrap();

    assert!(!text.contains("rfkill refused"), "{text}");
    assert!(!text.contains("error:"), "{text}");
}

#[test]
fn an_unread_router_is_not_none_and_leaves_no_gap_for_its_error() {
    let mut out = Vec::new();
    status(
        Read { json: false },
        &NoRoute,
        &mut Out::new(&mut out, Settings::default()),
    );
    let text = String::from_utf8(out).unwrap();

    assert!(text.contains("router     not read\n"), "{text}");
    assert!(!text.contains("\n\n\n"), "{text}");
    assert!(!text.contains("error:"), "{text}");
}

#[test]
fn json_stays_json_when_a_read_fails() {
    let mut out = Vec::new();
    status(
        Read { json: true },
        &Broken,
        &mut Out::new(&mut out, Settings::default()),
    );

    serde_json::from_slice::<serde_json::Value>(&out).unwrap();
}

/// A named field tells a script what the value is, as every other read
/// does.
#[test]
fn the_public_address_is_an_object_under_json() {
    let address = std::net::IpAddr::from([192, 0, 2, 1]);

    assert_eq!(
        address_shown(address, true),
        "{\n  \"schema\": 1,\n  \"address\": \"192.0.2.1\"\n}\n"
    );
    assert_eq!(address_shown(address, false), "192.0.2.1\n");
}
