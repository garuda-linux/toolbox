//! Boot Repair from the terminal of the live ISO. The same facts, the
//! same refusals and the same preflight as the page: a repair never
//! touches a disk the running system uses.

#[cfg(test)]
mod tests;

use std::io::Write as _;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{BootRepairCommand, Read, RepairArgs};
use toolbox_cli::confirm::Prompt;
use toolbox_cli::output::Row;
use toolbox_core::Toggle;
use toolbox_core::features::bootrepair::page::{Booted, Offer, target as chosen_target, targets};
use toolbox_core::features::bootrepair::{
    Encryption, Firmware, Refusal, RepairFacts, Target, Wish, plan_regenerate, plan_reinstall,
};
use toolbox_plan::{Password, Plan, VolumeDevice};
use toolbox_system::{Host, RepairSource as _, SystemRepair};

use super::act::run;
use super::change::{Refused, why};
use super::out::{Listing, failure};
use super::terminal::{Terminal, without_line_end};
use super::{json, system};
use crate::out::Out;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SystemRow {
    #[serde(rename = "id")]
    pub device: String,
    #[serde(rename = "name")]
    pub label: String,
    pub encryption: &'static str,
    pub in_use: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Status {
    pub offered: bool,
    pub firmware: &'static str,
    pub systems: Vec<SystemRow>,
    pub targets: Vec<String>,
}

pub fn command(command: BootRepairCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let live = system().part::<Host>().is_some_and(|host| host.live_iso);
    let facts = match SystemRepair::system().facts() {
        Ok(facts) => facts,
        Err(error) => {
            failure(out, "the disks were not read", &error.to_string());
            return Exit::Unavailable;
        }
    };

    match command {
        BootRepairCommand::Show(read) => status(read, live, &facts, out),
        BootRepairCommand::Regenerate(args) => {
            let planned = offered(live, &args).and_then(|()| {
                repair(
                    &facts,
                    &args,
                    None,
                    &mut Terminal {
                        color: out.settings().color,
                    },
                )
            });

            run(planned, args.confirm, None, runtime, out)
        }
        BootRepairCommand::Reinstall(args) => {
            let planned = offered(live, &args.repair).and_then(|()| {
                let target = pick_target(&facts, &args.repair.system, args.target.as_deref())?;
                repair(
                    &facts,
                    &args.repair,
                    Some(&target),
                    &mut Terminal {
                        color: out.settings().color,
                    },
                )
            });

            run(planned, args.repair.confirm, None, runtime, out)
        }
    }
}

/// # Errors
///
/// [`Refused`] for an unknown system or target, a locked system without
/// its passphrase, or what the page refuses.
pub fn repair(
    facts: &RepairFacts,
    args: &RepairArgs,
    target: Option<&Target>,
    prompt: &mut dyn Prompt,
) -> Result<Plan, Refused> {
    let found = facts.inventory.system(&args.system).ok_or_else(|| {
        Refused::Unknown(format!(
            "`{}` is no installed system here, see `bootrepair show`",
            args.system
        ))
    })?;

    let device = VolumeDevice::new(found.device())
        .map_err(|_invalid| Refused::Rule(format!("`{}` is no device", args.system)))?;

    let passphrase = if found.encryption == Encryption::Locked {
        Some(passphrase(args, prompt)?)
    } else {
        None
    };

    let wish = Wish {
        system: device,
        layout: args.layout,
        passphrase,
    };

    match target {
        Some(target) => plan_reinstall(facts, wish, target),
        None => plan_regenerate(facts, wish),
    }
    .map_err(|refusal| Refused::page_rule(why::repair(refusal)))
}

/// # Errors
///
/// [`Refused::Unknown`] when neither is a target the page lists.
pub fn pick_target(
    facts: &RepairFacts,
    system: &str,
    asked: Option<&str>,
) -> Result<Target, Refused> {
    let found = facts.inventory.system(system);
    let picked = asked.and_then(|device| {
        targets(facts)
            .into_iter()
            .find(|choice| choice.disk.as_str() == device || choice.target.device() == device)
            .map(|choice| choice.target)
    });

    if asked.is_some() && picked.is_none() {
        return Err(Refused::Unknown(
            "no such boot target here, see `bootrepair show`".to_owned(),
        ));
    }

    chosen_target(facts, picked.as_ref(), found)
        .map(|choice| choice.target)
        .ok_or_else(|| Refused::Unknown("no boot target found, pass --target".to_owned()))
}

/// As the page: offered on the live ISO, or with `--boot-repair` for a
/// test, which also wants `--trust-disk`.
fn offered(live: bool, args: &RepairArgs) -> Result<(), Refused> {
    let offer = Offer::of(Booted::of_live(live), Toggle::from_on(args.boot_repair));

    match (offer, args.trust_disk) {
        (Offer::Live, _) | (Offer::Test, true) => Ok(()),
        (Offer::Hidden, _) => Err(Refused::Rule(format!(
            "Boot Repair runs on the live ISO, pass --boot-repair to test it ({})",
            Refusal::NotOffered.as_str()
        ))),
        (Offer::Test, false) => Err(Refused::Rule(
            "outside the live ISO the chosen system's own programs run as root\n\
             pass --trust-disk if nobody else could write to that disk"
                .to_owned(),
        )),
    }
}

fn passphrase(args: &RepairArgs, prompt: &mut dyn Prompt) -> Result<Password, Refused> {
    let text = if args.passphrase_stdin {
        prompt.line()
    } else {
        prompt.secret(&format!("Passphrase of {}: ", args.system))
    };

    let text = text.ok_or_else(|| {
        Refused::Rule(
            "the system is locked and no passphrase was given: without a terminal to ask on, \
             pass --passphrase-stdin"
                .to_owned(),
        )
    })?;

    Password::new(without_line_end(text))
        .map_err(|invalid| Refused::Rule(format!("the passphrase is not usable: {invalid}")))
}

fn status(read: Read, live: bool, facts: &RepairFacts, out: &mut Out<'_>) -> Exit {
    let view = Status {
        offered: live,
        firmware: match facts.firmware {
            Firmware::Uefi => "uefi",
            Firmware::Bios => "bios",
        },
        systems: facts
            .inventory
            .systems
            .iter()
            .map(|system| SystemRow {
                device: system.device().to_owned(),
                label: system.label.clone(),
                encryption: system.encryption.as_str(),
                in_use: system.in_use,
            })
            .collect(),
        targets: targets(facts)
            .iter()
            .map(|choice| choice.target.device().to_owned())
            .collect(),
    };

    if read.json {
        write!(out, "{}", json(&view)).ok();
        return Exit::Success;
    }

    let paint = out.paint();
    let live = if live { "yes" } else { "no" };
    let summary = paint.facts(
        "Boot repair",
        &[("firmware", view.firmware), ("live ISO", live)],
    );

    let systems = system_listing(&view.systems);
    let targets: Vec<Row> = view
        .targets
        .iter()
        .map(|target| Row::plain(vec![target.clone()]))
        .collect();

    let count = targets.len().to_string();
    write!(
        out,
        "{summary}\n{}\n{}",
        paint.list(
            systems.title,
            systems.summary.as_deref(),
            systems.head,
            &systems.rows
        ),
        paint.list("Boot targets", Some(&count), &[], &targets),
    )
    .ok();

    if let Some(hint) = systems.hint {
        writeln!(out, "\n{}", paint.hint(&hint)).ok();
    }

    Exit::Success
}

/// The installed systems. The columns need their names, as a state is
/// either in use or the kind of encryption.
pub fn system_listing(rows: &[SystemRow]) -> Listing<'_> {
    let in_use = rows.iter().filter(|row| row.in_use).count();

    Listing {
        title: "Systems",
        summary: Some(format!("{}, {in_use} in use", rows.len())),
        head: &["DEVICE", "SYSTEM", "STATE"],
        rows: rows
            .iter()
            .map(|row| {
                let state = if row.in_use { "in use" } else { row.encryption };
                Row::plain(vec![
                    row.device.clone(),
                    row.label.clone(),
                    state.to_owned(),
                ])
            })
            .collect(),
        hint: (in_use < rows.len())
            .then(|| "garuda-toolbox bootrepair regenerate <device>".to_owned()),
    }
}
