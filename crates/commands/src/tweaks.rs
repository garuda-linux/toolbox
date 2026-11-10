//! `tweaks`: the System Tweaks page's DNS servers, Wi-Fi backend, hblock
//! and garuda-update options, each queued as the page queues it.

#[cfg(test)]
mod tests;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{ReadList, TweaksCommand};
use toolbox_cli::output::{Mark, Row};
use toolbox_core::domains::{
    CurrentDns, CurrentWifiBackend, Dns, GarudaUpdateConfig, GarudaUpdateFlags, Hblock,
    HblockState, WifiBackends,
};
use toolbox_core::{ChangeSet, Single, Snapshot, Toggle};
use toolbox_plan::GarudaUpdateFlag;

use crate::act::run;
use crate::change::fresh;
use crate::out::{Listing, Out};
use crate::{filter, system};

/// What a part reads as before its probe delivered it.
const NOT_READ: &str = "not read";

#[derive(Clone, Serialize)]
struct TweakRow {
    id: String,
    value: String,
}

impl filter::Named for TweakRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.id]
    }
}

pub fn command(command: &TweaksCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let snapshot = system();
    let confirm = match command {
        TweaksCommand::List(read) => return list(read, &snapshot, out),
        TweaksCommand::Dns(args) => args.confirm,
        TweaksCommand::WifiBackend(args) => args.confirm,
        TweaksCommand::Hblock(args) => args.confirm,
        TweaksCommand::UpdateFlag(args) => args.confirm,
    };

    let planned = fresh(|changes| {
        onto(command, &snapshot, changes);
        Ok(())
    });

    run(planned, confirm, None, runtime, out)
}

/// The page's wish behind `command`, onto `changes`. A wish the system
/// already fulfils is not queued, as on the page. `list` wishes nothing.
pub fn onto(command: &TweaksCommand, snapshot: &Snapshot, changes: &mut ChangeSet) {
    match command {
        TweaksCommand::List(_) => {}
        TweaksCommand::Dns(args) => changes.want::<Dns>(Single, args.provider, snapshot),
        TweaksCommand::WifiBackend(args) => {
            changes.want::<WifiBackends>(Single, args.backend, snapshot);
        }
        TweaksCommand::Hblock(args) => {
            changes.want::<Hblock>(Single, Toggle::from_on(args.state.on()), snapshot);
        }
        TweaksCommand::UpdateFlag(args) => {
            changes.want::<GarudaUpdateFlags>(
                args.flag,
                Toggle::from_on(args.state.on()),
                snapshot,
            );
        }
    }
}

fn rows(snapshot: &Snapshot) -> Vec<TweakRow> {
    let row = |id: &str, value: Option<&str>| TweakRow {
        id: id.to_owned(),
        value: value.unwrap_or(NOT_READ).to_owned(),
    };

    let on_off = |on: bool| if on { "on" } else { "off" };
    let flags = snapshot.part::<GarudaUpdateConfig>();

    [
        row(
            "dns",
            snapshot.part::<CurrentDns>().map(|dns| dns.0.as_str()),
        ),
        row(
            "wifi-backend",
            snapshot
                .part::<CurrentWifiBackend>()
                .map(|backend| backend.0.as_str()),
        ),
        row(
            "hblock",
            snapshot.part::<HblockState>().map(|state| on_off(state.0)),
        ),
    ]
    .into_iter()
    .chain(GarudaUpdateFlag::ALL.into_iter().map(|flag| {
        row(
            &format!("update-flag {}", flag.as_str()),
            flags.map(|config| on_off(config.0.contains(&flag))),
        )
    }))
    .collect()
}

fn list(read: &ReadList, snapshot: &Snapshot, out: &mut Out<'_>) -> Exit {
    filter::print_all(read, &rows(snapshot), out, |rows| Listing {
        title: "System Tweaks",
        summary: None,
        head: &[],
        rows: rows
            .iter()
            .map(|row| {
                let mark = match row.value.as_str() {
                    "off" => Mark::Off,
                    _ => Mark::On,
                };

                Row::new(mark, vec![row.id.clone(), row.value.clone()])
            })
            .collect(),
        hint: Some("garuda-toolbox tweaks dns cloudflare".to_owned()),
    })
}
