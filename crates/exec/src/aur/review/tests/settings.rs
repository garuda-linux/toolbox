use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::support::{Fetch, OCTOPI, case, commit, git_program, marker_program};
use crate::Environment;
use crate::aur::review::git::{EMPTY_TREE, invocation};
use crate::capture::{Policy, capture};

#[test]
fn every_git_run_starts_with_the_hardening_flags() {
    let (argv, _) = invocation(&["diff", "--quiet"]);

    assert_eq!(
        argv,
        [
            "--no-pager",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "diff",
            "--quiet",
        ]
    );
}

#[test]
fn git_runs_with_the_user_and_system_config_switched_off() {
    let (_, variables) = invocation(&[]);

    assert_eq!(
        variables,
        [
            ("GIT_CONFIG_GLOBAL", "/dev/null"),
            ("GIT_CONFIG_NOSYSTEM", "1"),
            ("GIT_ATTR_NOSYSTEM", "1"),
        ]
    );

    let names: Vec<&str> = Environment::Clean
        .variables()
        .iter()
        .chain(&variables)
        .map(|(name, _)| *name)
        .collect();

    for absent in ["HOME", "XDG_CONFIG_HOME", "GIT_CONFIG_COUNT"] {
        assert!(!names.contains(&absent), "{absent}");
    }

    assert!(!names.iter().any(|name| name.starts_with("GIT_CONFIG_KEY_")));
}

/// A plain `git diff` with no hardening flag: only the variables decide
/// whether the user's `diff.external` runs. The builder's variables keep it
/// out, and the same command with `GIT_CONFIG_GLOBAL` pointing at a file that
/// names the marker program runs it.
#[test]
fn only_git_config_global_lets_a_user_diff_program_run() {
    let case = case(Fetch::Copy);
    let source = case.sources.join(OCTOPI);
    let head = commit(&source, &[("PKGBUILD", "pkgname=octopi-git\npkgver=2\n")]);
    let (program, marker) = marker_program(&case.sources, "user-external");
    let user_config = case.sources.join("user.gitconfig");
    fs::write(
        &user_config,
        format!("[diff]\n\texternal = {}\n", program.display()),
    )
    .unwrap();

    let (_, builder) = invocation(&[]);
    let replaced: Vec<(&str, &str)> = builder
        .iter()
        .map(|(name, value)| match *name {
            "GIT_CONFIG_GLOBAL" => (*name, user_config.to_str().unwrap()),
            _ => (*name, *value),
        })
        .collect();

    let run = |variables: &[(&str, &str)]| {
        let mut command = Command::new(git_program());
        command
            .current_dir(&source)
            .args(["--no-pager", "diff", EMPTY_TREE, head.as_str(), "--"]);

        capture(
            &mut command,
            Environment::Clean,
            variables,
            &Policy::whole(),
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    };

    run(&builder);
    assert!(
        !marker.exists(),
        "the builder's variables let the user's config run"
    );

    run(&replaced);
    assert!(marker.exists(), "the user's config did not run");
}
