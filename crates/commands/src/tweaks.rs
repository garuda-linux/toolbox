//! `tweaks`: the System Tweaks page's DNS servers, Wi-Fi backend, hblock
//! and garuda-update options, each queued as the page queues it.

#[cfg(test)]
mod tests;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{ReadList, TweaksCommand};
use toolbox_cli::output::{Mark, Row};
use toolbox_core::features::tweaks::view::{Choices, Flip, Pick};
use toolbox_core::features::tweaks::{self, Tweak};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
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
    #[serde(skip)]
    mark: Mark,
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
    let tweak = match command {
        TweaksCommand::List(_) => return,
        TweaksCommand::Dns(args) => Tweak::Dns(args.provider),
        TweaksCommand::WifiBackend(args) => Tweak::Wifi(args.backend),
        TweaksCommand::Hblock(args) => Tweak::Hblock(Toggle::from_on(args.state.on())),
        TweaksCommand::UpdateFlag(args) => Tweak::Flag(args.flag, Toggle::from_on(args.state.on())),
    };

    tweaks::queue(changes, tweak, snapshot);
}

/// What the page shows with nothing queued.
fn rows(snapshot: &Snapshot) -> Vec<TweakRow> {
    let choices = Choices::of(snapshot, &ChangeSet::default(), None);

    let row = |id: &str, shown: Option<(Mark, &str)>| match shown {
        Some((mark, value)) => TweakRow {
            id: id.to_owned(),
            value: value.to_owned(),
            mark,
        },
        None => TweakRow {
            id: id.to_owned(),
            value: NOT_READ.to_owned(),
            mark: Mark::Unknown,
        },
    };

    let on_off = |flip: Flip| {
        flip.known.then_some(if flip.on {
            (Mark::On, "on")
        } else {
            (Mark::Off, "off")
        })
    };

    let flag_state = |flag: GarudaUpdateFlag| {
        let flags = choices.flags.as_deref()?;
        let (_, flip) = flags.iter().find(|(listed, _)| *listed == flag)?;
        on_off(*flip)
    };

    [
        row(
            "dns",
            shown(&choices.dns).map(|dns| (Mark::Blank, dns.as_str())),
        ),
        row(
            "wifi-backend",
            shown(&choices.wifi).map(|backend| (Mark::Blank, backend.as_str())),
        ),
        row("hblock", on_off(choices.hblock)),
    ]
    .into_iter()
    .chain(
        GarudaUpdateFlag::ALL
            .into_iter()
            .map(|flag| row(&format!("update-flag {}", flag.as_str()), flag_state(flag))),
    )
    .collect()
}

fn shown<T>(pick: &Pick<T>) -> Option<&T> {
    pick.options.get(pick.shown?)
}

fn list(read: &ReadList, snapshot: &Snapshot, out: &mut Out<'_>) -> Exit {
    filter::print_all(read, &rows(snapshot), out, |rows| Listing {
        title: "System Tweaks",
        summary: None,
        head: &[],
        rows: rows
            .iter()
            .map(|row| Row::new(row.mark, vec![row.id.clone(), row.value.clone()]))
            .collect(),
        hint: Some("garuda-toolbox tweaks dns cloudflare".to_owned()),
    })
}
