use std::path::PathBuf;

use toolbox_cli::output::{Paint, Style};
use toolbox_system::CallError;

use super::facts::{CatalogFacts, LogFacts, PROGRAMS, Pacman, ProgramFacts, Tool, agent};
use super::{Check, Facts, State, checks, needs_fixing, text};

fn tool(path: &str) -> Tool {
    Tool {
        path: PathBuf::from(path),
        present: true,
        setuid_root: true,
    }
}

fn healthy() -> Facts {
    Facts {
        development: false,
        dry_run: false,
        pkexec: tool("/usr/bin/pkexec"),
        sudo: tool("/usr/bin/sudo"),
        agent: Some("polkit-kde-auth".to_owned()),
        helper: PathBuf::from("/usr/lib/garuda-toolbox/garuda-toolbox-helper"),
        helper_present: true,
        policy: Some(PathBuf::from(
            "/usr/share/polkit-1/actions/org.garudalinux.toolbox.policy",
        )),
        pacman: Pacman {
            local: true,
            repositories: 4,
        },
        programs: ProgramFacts {
            installed: PROGRAMS.to_vec(),
        },
        system_bus: Ok(()),
        user_bus: Ok(()),
        catalogs: CatalogFacts {
            dir: PathBuf::from("/usr/share/garuda-toolbox/catalog"),
            loaded: 5,
            problems: Vec::new(),
        },
        log: LogFacts {
            file: Some(PathBuf::from(
                "/home/ada/.local/state/garuda-toolbox/garuda-toolbox.log",
            )),
            writable: true,
        },
    }
}

fn find<'a>(checks: &'a [Check], name: &str) -> &'a Check {
    checks
        .iter()
        .find(|check| check.name == name)
        .unwrap_or_else(|| panic!("no check {name}"))
}

#[test]
fn a_healthy_system_needs_nothing() {
    let found = checks(&healthy());

    assert!(
        found.iter().all(|check| check.state == State::Ok),
        "{found:#?}"
    );
    assert!(found.iter().all(|check| check.hint.is_none()));
    assert!(!needs_fixing(&found));
}

#[test]
fn every_part_the_brief_names_is_checked() {
    let found = checks(&healthy());
    for name in [
        "helper",
        "pkexec",
        "polkit agent",
        "sudo",
        "polkit policy",
        "pacman databases",
        "checkupdates",
        "paru",
        "garuda-update",
        "garuda-health",
        "snapper",
        "system bus",
        "user bus",
        "catalogs",
        "log folder",
    ] {
        find(&found, name);
    }
}

#[test]
fn a_pkexec_without_setuid_is_a_problem_with_a_reinstall_hint() {
    let mut facts = healthy();
    facts.pkexec.setuid_root = false;
    let found = checks(&facts);

    let pkexec = find(&found, "pkexec");
    assert_eq!(pkexec.state, State::Problem);
    assert!(pkexec.text.contains("not setuid root"), "{}", pkexec.text);
    assert_eq!(
        pkexec.hint.as_deref(),
        Some("reinstall polkit: sudo pacman -S polkit")
    );
    assert!(needs_fixing(&found));
}

#[test]
fn without_pkexec_and_sudo_nothing_raises_the_helper() {
    let mut facts = healthy();
    facts.pkexec.present = false;
    facts.pkexec.setuid_root = false;
    facts.sudo.present = false;
    facts.sudo.setuid_root = false;
    let found = checks(&facts);

    assert_eq!(find(&found, "sudo").state, State::Problem);
    assert_eq!(
        find(&found, "pkexec").hint.as_deref(),
        Some("install polkit as root: pacman -S polkit")
    );
}

#[test]
fn a_missing_sudo_is_only_a_note_while_pkexec_works() {
    let mut facts = healthy();
    facts.sudo.present = false;
    facts.sudo.setuid_root = false;

    assert_eq!(find(&checks(&facts), "sudo").state, State::Optional);
}

#[test]
fn no_polkit_agent_found_is_unknown_not_a_problem() {
    let mut facts = healthy();
    facts.agent = None;
    let found = checks(&facts);

    assert_eq!(find(&found, "polkit agent").state, State::Unknown);
    assert!(!needs_fixing(&found));
}

#[test]
fn a_missing_helper_names_the_fix_of_the_build() {
    let mut facts = healthy();
    facts.helper_present = false;
    assert_eq!(
        find(&checks(&facts), "helper").hint.as_deref(),
        Some("reinstall the toolbox: sudo pacman -S garuda-toolbox")
    );

    facts.development = true;
    assert_eq!(
        find(&checks(&facts), "helper").hint.as_deref(),
        Some("build it: cargo build --bin garuda-toolbox-helper")
    );
}

#[test]
fn a_missing_policy_is_a_note_in_a_development_build_only() {
    let mut facts = healthy();
    facts.policy = None;
    assert_eq!(find(&checks(&facts), "polkit policy").state, State::Problem);

    facts.development = true;
    assert_eq!(
        find(&checks(&facts), "polkit policy").state,
        State::Optional
    );
}

#[test]
fn optional_programs_are_notes_and_checkupdates_is_needed() {
    let mut facts = healthy();
    facts.programs.installed.clear();
    let found = checks(&facts);

    let checkupdates = find(&found, "checkupdates");
    assert_eq!(checkupdates.state, State::Problem);
    assert_eq!(
        checkupdates.hint.as_deref(),
        Some("install pacman-contrib: sudo pacman -S pacman-contrib")
    );

    for name in ["paru", "garuda-update", "garuda-health", "snapper"] {
        let check = find(&found, name);
        assert_eq!(check.state, State::Optional, "{name}");
        assert!(check.text.starts_with("not installed"), "{}", check.text);
    }
}

#[test]
fn unreadable_pacman_databases_and_no_repositories_are_problems() {
    let mut facts = healthy();
    facts.pacman.repositories = 0;
    let none = checks(&facts);
    assert_eq!(find(&none, "pacman databases").state, State::Problem);
    assert_eq!(
        find(&none, "pacman databases").hint.as_deref(),
        Some("sync them with a full update: garuda-toolbox update")
    );

    facts.pacman.local = false;
    assert_eq!(
        find(&checks(&facts), "pacman databases").state,
        State::Problem
    );
}

#[test]
fn an_unreachable_bus_says_why() {
    let mut facts = healthy();
    facts.user_bus = Err(CallError::Unavailable);
    let found = checks(&facts);

    let user = find(&found, "user bus");
    assert_eq!(user.state, State::Problem);
    assert!(user.text.contains("not available on this system"));
    assert_eq!(find(&found, "system bus").state, State::Ok);
}

#[test]
fn a_broken_catalog_and_an_unwritable_log_are_problems() {
    let mut facts = healthy();
    facts.catalogs.problems = vec!["gaming.toml: missing field `id`".to_owned()];
    facts.log.writable = false;
    let found = checks(&facts);

    let catalogs = find(&found, "catalogs");
    assert_eq!(catalogs.state, State::Problem);
    assert!(catalogs.text.contains("gaming.toml"), "{}", catalogs.text);
    assert_eq!(find(&found, "log folder").state, State::Problem);

    facts.log.file = None;
    assert_eq!(find(&checks(&facts), "log folder").state, State::Problem);
}

#[test]
fn the_text_lists_every_check_and_a_hint_per_problem() {
    let mut facts = healthy();
    facts
        .programs
        .installed
        .retain(|name| !["checkupdates", "paru"].contains(name));
    let shown = text(&checks(&facts), Paint::new(Style::Plain));

    assert!(
        shown.starts_with("Doctor  (13 ok, 1 problem, 1 not installed)\n"),
        "{shown}"
    );
    assert!(shown.contains("  checkupdates"));
    assert!(shown.contains("hint: checkupdates: install pacman-contrib"));
    assert!(!shown.contains("hint: paru"), "{shown}");
}

#[test]
fn the_json_names_each_result() {
    let mut facts = healthy();
    facts.agent = None;
    let json = toolbox_cli::output::json(&checks(&facts));

    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let agent = &parsed["rows"][2];

    assert_eq!(parsed["schema"], 1_u64);
    assert_eq!(agent["id"], "polkit-agent");
    assert_eq!(agent["name"], "polkit agent");
    assert_eq!(agent["result"], "unknown");
    assert!(json.contains("\"result\": \"ok\""), "{json}");
    assert!(!json.contains("\"state\""), "{json}");
}

/// Scripts pick a check by its id, so the ids stay as they are, whatever
/// the English name says.
#[test]
fn every_check_has_a_stable_kebab_case_id() {
    let ids: Vec<&str> = checks(&healthy()).iter().map(|check| check.id).collect();

    assert_eq!(
        ids,
        [
            "helper",
            "pkexec",
            "polkit-agent",
            "sudo",
            "polkit-policy",
            "pacman-databases",
            "checkupdates",
            "paru",
            "garuda-update",
            "garuda-health",
            "snapper",
            "system-bus",
            "user-bus",
            "catalogs",
            "log-folder",
        ]
    );
}

#[test]
fn the_ids_are_kebab_case_and_told_apart() {
    let found = checks(&healthy());
    let ids: std::collections::BTreeSet<&str> = found.iter().map(|check| check.id).collect();

    assert_eq!(ids.len(), found.len());
    assert!(ids.iter().all(|id| {
        !id.is_empty()
            && !id.starts_with('-')
            && !id.ends_with('-')
            && id.chars().all(|letter| {
                letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '-'
            })
    }));
}

#[test]
fn a_polkit_agent_is_found_by_its_process_name() {
    let names = |list: &[&str]| {
        list.iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>()
    };

    assert_eq!(
        agent(&names(&["bash", "polkit-kde-auth"])).as_deref(),
        Some("polkit-kde-auth")
    );
    assert_eq!(
        agent(&names(&["gnome-shell"])).as_deref(),
        Some("gnome-shell")
    );
    assert_eq!(
        agent(&names(&["lxqt-policykit-"])).as_deref(),
        Some("lxqt-policykit-")
    );
    assert_eq!(agent(&names(&["polkitd", "polkit-agent-he", "bash"])), None);
}
