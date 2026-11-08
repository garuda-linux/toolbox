//! The judgement of each fact: fine, a problem with its fix, or a note
//! about a feature that is left out. Pure, so every case has a test.

use std::path::Path;

use toolbox_system::CallError;

use super::facts::{Facts, Tool};
use super::{Check, Part, State};

const REINSTALL: &str = "reinstall the toolbox: sudo pacman -S garuda-toolbox";

const NO_AGENT: &str = "none found: the window may not be able to ask for the password, the command line asks in the terminal";

const START_AGENT: &str =
    "start your desktop's polkit agent, such as polkit-kde-agent or polkit-gnome";

const HELPER: Part = Part::named("helper");
const PKEXEC: Part = Part::named("pkexec");
const AGENT: Part = Part::new("polkit-agent", "polkit agent");
const SUDO: Part = Part::named("sudo");
const POLICY: Part = Part::new("polkit-policy", "polkit policy");
const PACMAN: Part = Part::new("pacman-databases", "pacman databases");
const CHECKUPDATES: Part = Part::named("checkupdates");
const CATALOGS: Part = Part::named("catalogs");
const LOG: Part = Part::new("log-folder", "log folder");

pub fn checks(facts: &Facts) -> Vec<Check> {
    let buses = [
        bus(
            Part::new("system-bus", "system bus"),
            &facts.system_bus,
            "systemd answers: system services, power and boot",
            "see why: systemctl status dbus.service",
        ),
        bus(
            Part::new("user-bus", "user bus"),
            &facts.user_bus,
            "your systemd answers: your own services",
            "see why: systemctl --user status",
        ),
    ];

    [
        helper(facts),
        pkexec(facts),
        agent(facts),
        sudo(facts),
        policy(facts),
        pacman(facts),
    ]
    .into_iter()
    .chain(programs(facts))
    .chain(buses)
    .chain([catalogs(facts), log(facts)])
    .collect()
}

const fn ok(part: Part, text: String) -> Check {
    Check {
        id: part.id,
        name: part.name,
        state: State::Ok,
        text,
        hint: None,
    }
}

const fn problem(part: Part, text: String, hint: String) -> Check {
    Check {
        id: part.id,
        name: part.name,
        state: State::Problem,
        text,
        hint: Some(hint),
    }
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

const fn raises(tool: &Tool) -> bool {
    tool.present && tool.setuid_root
}

/// A package install, through sudo when it works, else as root.
fn install(verb: &str, package: &str, facts: &Facts) -> String {
    if raises(&facts.sudo) {
        format!("{verb} {package}: sudo pacman -S {package}")
    } else {
        format!("{verb} {package} as root: pacman -S {package}")
    }
}

fn helper(facts: &Facts) -> Check {
    let path = shown(&facts.helper);
    if !facts.helper_present {
        let hint = if facts.development {
            "build it: cargo build --bin garuda-toolbox-helper"
        } else {
            REINSTALL
        };

        return problem(
            HELPER,
            format!("{path} is missing: no change and no root read can run"),
            hint.to_owned(),
        );
    }

    if facts.dry_run {
        ok(
            HELPER,
            format!("{path} in dry-run mode: changes are only checked"),
        )
    } else {
        ok(HELPER, format!("{path}: changes and root reads can run"))
    }
}

fn pkexec(facts: &Facts) -> Check {
    let path = shown(&facts.pkexec.path);
    if raises(&facts.pkexec) {
        return ok(
            PKEXEC,
            format!("{path}, setuid root: the window and the command line reach the helper"),
        );
    }

    if facts.pkexec.present {
        problem(
            PKEXEC,
            format!("{path} is not setuid root: the window cannot change the system"),
            install("reinstall", "polkit", facts),
        )
    } else {
        problem(
            PKEXEC,
            format!("{path} is missing: the window cannot change the system"),
            install("install", "polkit", facts),
        )
    }
}

/// Only a hint of a check: an agent may run under a name the list does not
/// know, and the command line asks in the terminal without one.
fn agent(facts: &Facts) -> Check {
    facts.agent.as_ref().map_or_else(
        || Check {
            id: AGENT.id,
            name: AGENT.name,
            state: State::Unknown,
            text: NO_AGENT.to_owned(),
            hint: Some(START_AGENT.to_owned()),
        },
        |name| {
            ok(
                AGENT,
                format!("{name} runs: the window can ask for the password"),
            )
        },
    )
}

fn sudo(facts: &Facts) -> Check {
    let path = shown(&facts.sudo.path);
    if raises(&facts.sudo) {
        return ok(
            SUDO,
            format!("{path}: the command line's fallback when pkexec cannot ask"),
        );
    }

    let what = if facts.sudo.present {
        format!("{path} is not setuid root")
    } else {
        "not installed".to_owned()
    };

    if raises(&facts.pkexec) {
        Check {
            id: SUDO.id,
            name: SUDO.name,
            state: State::Optional,
            text: format!("{what}: no fallback when pkexec cannot ask"),
            hint: Some("install sudo: pkexec pacman -S sudo".to_owned()),
        }
    } else {
        problem(
            SUDO,
            format!("{what} and pkexec cannot raise: nothing can start the helper as root"),
            "install sudo as root: pacman -S sudo".to_owned(),
        )
    }
}

fn policy(facts: &Facts) -> Check {
    if let Some(file) = &facts.policy {
        return ok(
            POLICY,
            format!("{}: polkit knows the toolbox's actions", shown(file)),
        );
    }

    if facts.development {
        Check {
            id: POLICY.id,
            name: POLICY.name,
            state: State::Optional,
            text: "not installed: a development build asks through pkexec's own action every time"
                .to_owned(),
            hint: Some(REINSTALL.to_owned()),
        }
    } else {
        problem(
            POLICY,
            "not installed: polkit does not know the toolbox's actions".to_owned(),
            REINSTALL.to_owned(),
        )
    }
}

fn pacman(facts: &Facts) -> Check {
    if !facts.pacman.local {
        return problem(
            PACMAN,
            "the local database cannot be read: packages, kernels and apps show nothing".to_owned(),
            "the toolbox needs pacman and /var/lib/pacman/local readable".to_owned(),
        );
    }

    match facts.pacman.repositories {
        0 => problem(
            PACMAN,
            "no repository database: updates and the app catalog cannot be read".to_owned(),
            "sync them with a full update: garuda-toolbox update".to_owned(),
        ),
        1 => ok(
            PACMAN,
            "readable, 1 repository: packages, kernels and apps".to_owned(),
        ),
        many => ok(
            PACMAN,
            format!("readable, {many} repositories: packages, kernels and apps"),
        ),
    }
}

/// The programs some features need. Only checkupdates is a dependency of
/// the toolbox, the others are optional.
fn programs(facts: &Facts) -> Vec<Check> {
    let found = &facts.programs;
    let checkupdates = if found.has("checkupdates") {
        ok(
            CHECKUPDATES,
            "installed: status and Home count the updates".to_owned(),
        )
    } else {
        problem(
            CHECKUPDATES,
            "missing: status and Home cannot count the updates".to_owned(),
            install("install", "pacman-contrib", facts),
        )
    };

    let optional = [
        (
            found.has("paru"),
            "paru",
            "AUR packages and their updates",
            "no AUR packages and no AUR updates",
        ),
        (
            found.has("garuda-update"),
            "garuda-update",
            "updates the Garuda way and the Rescue repairs",
            "updates run through pacman, without the Rescue repairs",
        ),
        (
            found.has("garuda-health"),
            "garuda-health",
            "the health findings of status and Home",
            "status and Home show no health findings",
        ),
        (
            found.has("snapper"),
            "snapper",
            "the snapshot settings among the config files",
            "no snapshot settings among the config files",
        ),
    ];

    let mut checks = vec![checkupdates];
    checks.extend(optional.into_iter().map(|(present, name, with, without)| {
        if present {
            ok(Part::named(name), format!("installed: {with}"))
        } else {
            Check {
                id: name,
                name,
                state: State::Optional,
                text: format!("not installed: {without}"),
                hint: Some(install("install", name, facts)),
            }
        }
    }));
    checks
}

fn bus(part: Part, reached: &Result<(), CallError>, fine: &str, hint: &str) -> Check {
    match reached {
        Ok(()) => ok(part, fine.to_owned()),
        Err(reason) => problem(
            part,
            format!("systemd does not answer: {reason}"),
            hint.to_owned(),
        ),
    }
}

fn catalogs(facts: &Facts) -> Check {
    let found = &facts.catalogs;
    let dir = shown(&found.dir);
    let fix = if facts.development {
        "fix the file, then check it: cargo xtask catalog-validate"
    } else {
        REINSTALL
    };

    found.problems.first().map_or_else(
        || ok(CATALOGS, format!("{} loaded from {dir}", found.loaded)),
        |first| {
            problem(
                CATALOGS,
                format!("{} broken in {dir}: {first}", found.problems.len()),
                fix.to_owned(),
            )
        },
    )
}

fn log(facts: &Facts) -> Check {
    match (&facts.log.file, facts.log.writable) {
        (Some(file), true) => ok(
            LOG,
            format!("{}: the log of every run goes there", shown(file)),
        ),
        (Some(file), false) => {
            let folder = file.parent().map_or_else(String::new, shown);
            problem(
                LOG,
                format!(
                    "{} cannot be written: the log goes to stderr alone",
                    shown(file)
                ),
                format!("make the folder yours: sudo chown -R $USER {folder}"),
            )
        }
        (None, _) => problem(
            LOG,
            "no safe folder: the log goes to stderr alone".to_owned(),
            "set HOME, or XDG_STATE_HOME to a folder of yours".to_owned(),
        ),
    }
}
