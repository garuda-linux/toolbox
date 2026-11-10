#[cfg(test)]
mod tests;

use std::io::Write as _;
use std::net::IpAddr;
use std::sync::Arc;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{
    Confirm, DriversCommand, ModuleArgs, NetworkCommand, Radio, Read, ReadList,
};
use toolbox_cli::output::{Mark, Row};
use toolbox_core::features::network::{
    ModuleFacts, RadioKind, RadioTarget, driver_rows, plan_ping, plan_traceroute, set_blacklisted,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{Bounded, HostName, Plan};
use toolbox_system::{
    AddressProblem, KdeGeoIp, NetworkActions as _, NetworkSource, SystemActions, SystemNetwork,
    lookup_address,
};

use super::act::run;
use super::change::{Refused, fresh, page_edit, why};
use super::out::{Listing, failure};
use super::{json, system, unavailable};
use crate::out::Out;

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct StatusView {
    pub gateway: Option<String>,
    pub interface: Option<String>,
    pub wifi: Option<bool>,
    pub bluetooth: Option<bool>,
    pub flight_mode: bool,
    /// The reads that failed, so a script tells "no router" from a broken
    /// read.
    pub unread: Vec<&'static str>,
    #[serde(skip)]
    pub reasons: Vec<(&'static str, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DriverLine {
    #[serde(rename = "id")]
    pub module: String,
    pub category: &'static str,
    pub loaded: bool,
    pub blacklisted: bool,
}

#[derive(Serialize)]
struct PublicAddress {
    address: String,
}

pub fn command(command: NetworkCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    match command {
        NetworkCommand::Show(read) => status(read, &SystemNetwork::system(), out),
        NetworkCommand::Radio(args) => radio(args.radio, args.state.on(), out),
        NetworkCommand::Drivers(DriversCommand::List(read)) => drivers(&read, out),
        NetworkCommand::Drivers(DriversCommand::Blacklist(args)) => {
            let planned = blacklist(&args, true, &system());
            run(planned, args.confirm, None, runtime, out)
        }
        NetworkCommand::Drivers(DriversCommand::Allow(args)) => {
            let planned = blacklist(&args, false, &system());
            run(planned, args.confirm, None, runtime, out)
        }
        NetworkCommand::PublicAddress(read) => public_address(read, out),
        NetworkCommand::Ping(args) => {
            let planned = ping(&args.host, args.count, args.timeout);
            tests(planned, runtime, out)
        }
        NetworkCommand::Traceroute(args) => {
            let planned = traceroute(&args.host, args.max_hops);
            tests(planned, runtime, out)
        }
    }
}

/// # Errors
///
/// [`Refused`] for a module that is no network driver here, or what the
/// page refuses.
pub fn blacklist(args: &ModuleArgs, on: bool, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| blacklist_onto(args, on, snapshot, changes))
}

/// # Errors
///
/// As [`blacklist`].
pub fn blacklist_onto(
    args: &ModuleArgs,
    on: bool,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let facts = snapshot
        .part::<ModuleFacts>()
        .ok_or_else(|| Refused::Rule("the drivers could not be read".to_owned()))?;

    page_edit(changes, snapshot, why::driver, |changes, snapshot| {
        set_blacklisted(&args.module, on, facts, changes, snapshot)
    })
}

/// # Errors
///
/// [`Refused::Rule`] for a number outside its bounds.
pub fn ping(host: &HostName, count: u32, timeout: u32) -> Result<Plan, Refused> {
    Ok(plan_ping(
        host.clone(),
        bounded(count, "count")?,
        bounded(timeout, "timeout")?,
    ))
}

/// # Errors
///
/// [`Refused::Rule`] for a number outside its bounds.
pub fn traceroute(host: &HostName, hops: u32) -> Result<Plan, Refused> {
    Ok(plan_traceroute(host.clone(), bounded(hops, "max-hops")?))
}

pub fn view(source: &dyn NetworkSource) -> StatusView {
    let mut unread = Vec::new();
    let mut reasons = Vec::new();
    let route = source
        .default_route()
        .map_err(|error| {
            unread.push("router");
            reasons.push(("the router could not be read", error.to_string()));
        })
        .ok();

    let radios = source
        .radios()
        .map_err(|error| {
            unread.push("radios");
            reasons.push(("the radios could not be read", error.to_string()));
        })
        .unwrap_or_default();

    let on = |kind: RadioKind| {
        let mut of_kind = radios
            .iter()
            .filter(|radio| radio.kind == Some(kind))
            .peekable();

        of_kind
            .peek()
            .is_some()
            .then(|| of_kind.all(|radio| !radio.soft && !radio.hard))
    };

    StatusView {
        gateway: route
            .as_ref()
            .and_then(|route| route.gateway.map(|ip| ip.to_string())),
        interface: route.and_then(|route| route.interface),
        wifi: on(RadioKind::Wifi),
        bluetooth: on(RadioKind::Bluetooth),
        flight_mode: !radios.is_empty() && radios.iter().all(|radio| radio.soft || radio.hard),
        unread,
        reasons,
    }
}

fn status(read: Read, source: &dyn NetworkSource, out: &mut Out<'_>) -> Exit {
    let view = super::spinner::while_reading(read.json || out.quiet(), || view(source));
    if read.json {
        write!(out, "{}", json(&view)).ok();
    } else {
        let paint = out.paint();
        let absent = if view.unread.contains(&"router") {
            "not read"
        } else {
            "none"
        };

        let facts = paint.facts(
            "Network",
            &[
                ("router", view.gateway.as_deref().unwrap_or(absent)),
                ("interface", view.interface.as_deref().unwrap_or(absent)),
            ],
        );

        let radios = radio_listing(&view);
        let summary = radios.summary.as_deref();
        let list = paint.list(radios.title, summary, radios.head, &radios.rows);
        write!(out, "{facts}\n{list}").ok();

        if let Some(hint) = radios.hint {
            writeln!(out, "\n{}", paint.hint(&hint)).ok();
        }

        for (what, raw) in &view.reasons {
            failure(out, what, raw);
        }
    }

    if view.unread.is_empty() {
        Exit::Success
    } else {
        Exit::Failure
    }
}

/// The switch runs at once through rfkill or NetworkManager, as the page
/// does, so it asks nothing.
fn radio(radio: Radio, on: bool, out: &mut Out<'_>) -> Exit {
    let (target, blocked) = match radio {
        Radio::Wifi => (RadioTarget::Wifi, !on),
        Radio::Bluetooth => (RadioTarget::Bluetooth, !on),
        Radio::Flight => (RadioTarget::All, on),
    };

    let source: Arc<dyn NetworkSource> = Arc::new(SystemNetwork::system());
    match SystemActions::new(source).set_radio(target, blocked) {
        Ok(()) => Exit::Success,
        Err(error) => {
            failure(out, "the radio was not switched", &error.to_string());
            Exit::Failure
        }
    }
}

fn drivers(read: &ReadList, out: &mut Out<'_>) -> Exit {
    let snapshot = system();
    let Some(facts) = snapshot.part::<ModuleFacts>() else {
        return unavailable("the drivers could not be read", out);
    };

    let rows: Vec<DriverLine> = driver_rows(facts, &ChangeSet::default())
        .into_iter()
        .map(|row| DriverLine {
            module: row.name.as_str().to_owned(),
            category: row.category.as_str(),
            loaded: row.loaded,
            blacklisted: row.own || !row.other_files.is_empty(),
        })
        .collect();

    super::filter::print_all(read, &rows, out, driver_listing)
}

pub fn driver_listing(rows: &[DriverLine]) -> Listing<'_> {
    let loaded = rows.iter().filter(|row| row.loaded).count();
    let blacklisted = rows.iter().filter(|row| row.blacklisted).count();

    Listing {
        title: "Network drivers",
        summary: Some(format!(
            "{}, {loaded} loaded, {blacklisted} blacklisted",
            rows.len()
        )),
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let (mark, state) = match (row.blacklisted, row.loaded) {
                    (true, _) => (Mark::Off, "blacklisted"),
                    (false, true) => (Mark::On, "loaded"),
                    (false, false) => (Mark::On, ""),
                };

                let cells = [row.module.as_str(), row.category, state];
                Row::new(mark, cells.map(str::to_owned).to_vec())
            })
            .collect(),
        hint: (!rows.is_empty())
            .then(|| "garuda-toolbox network drivers blacklist <module>".to_owned()),
    }
}

pub fn radio_listing(view: &StatusView) -> Listing<'static> {
    let read = !view.unread.contains(&"radios");
    let row = |name: &str, on: Option<bool>| {
        let (mark, state) = match (read, on) {
            (false, _) => (Mark::Unknown, "unknown"),
            (true, Some(true)) => (Mark::On, "on"),
            (true, Some(false)) => (Mark::Off, "off"),
            (true, None) => (Mark::Blank, "none"),
        };

        Row::new(mark, vec![name.to_owned(), state.to_owned()])
    };

    let rows = vec![
        row("wifi", view.wifi),
        row("bluetooth", view.bluetooth),
        row("flight", Some(view.flight_mode)),
    ];

    let on = rows.iter().filter(|radio| radio.mark == Mark::On).count();

    Listing {
        title: "Radios",
        summary: Some(format!("{on} on")),
        head: &[],
        rows,
        hint: read.then(|| "garuda-toolbox network radio <radio> on|off".to_owned()),
    }
}

fn public_address(read: Read, out: &mut Out<'_>) -> Exit {
    match lookup_address(&KdeGeoIp::new()) {
        Ok(address) => {
            write!(out, "{}", address_shown(address, read.json)).ok();
            Exit::Success
        }
        Err(problem) => {
            match problem {
                AddressProblem::NoAnswer(error) => {
                    failure(out, "the public address is not known", &error.to_string());
                }
                AddressProblem::NotUnderstood => crate::say_error(
                    out,
                    "the public address is not known: the answer held no address",
                ),
            }

            Exit::Failure
        }
    }
}

fn address_shown(address: IpAddr, as_json: bool) -> String {
    if as_json {
        json(&PublicAddress {
            address: address.to_string(),
        })
    } else {
        format!("{address}\n")
    }
}

/// The tests change nothing, so they run without a question.
fn tests(planned: Result<Plan, Refused>, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let confirm = Confirm {
        yes: true,
        dry_run: false,
        json: false,
    };

    run(planned, confirm, None, runtime, out)
}

fn bounded<const MIN: u32, const MAX: u32>(
    value: u32,
    what: &str,
) -> Result<Bounded<MIN, MAX>, Refused> {
    Bounded::new(i64::from(value))
        .map_err(|_out| Refused::Rule(format!("--{what} must lie between {MIN} and {MAX}")))
}
