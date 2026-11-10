use toolbox_plan::KernelRelease;

use super::{DkmsEntry, DkmsState, ModuleState, builds, modules, parse_status};

// `dkms status` as root in the Lima VM, dkms 3.x with two kernels. The
// steps between the four captures are not logged, only their output.
const INSTALLED: &str = crate::testing::kernels::dkms::INSTALLED_STDOUT;
const ONE_KERNEL_MISSING: &str = include_str!("fixtures/one-kernel-missing.stdout");
const ADDED: &str = crate::testing::kernels::dkms::ADDED_STDOUT;
const BROKEN: &str = crate::testing::kernels::dkms::BROKEN_STDOUT;
const BROKEN_STDERR: &str = crate::testing::kernels::dkms::BROKEN_STDERR;

const GARUDA: &str = "7.3.1-1-garuda";
const LTS: &str = "6.18.60-1-lts";

fn release(name: &str) -> KernelRelease {
    KernelRelease::new(name).unwrap()
}

fn shown(entries: &[DkmsEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{}/{} {} {}",
                entry.module,
                entry.version,
                entry.kernel.as_ref().map_or("-", KernelRelease::as_str),
                match entry.state {
                    DkmsState::Added => "added",
                    DkmsState::Built => "built",
                    DkmsState::Installed => "installed",
                    DkmsState::Broken => "broken",
                }
            )
        })
        .collect()
}

fn states(kernel: &str, output: &str) -> Vec<(String, ModuleState)> {
    modules(&release(kernel), &parse_status(output))
        .into_iter()
        .map(|module| (module.module.to_string(), module.state))
        .collect()
}

fn missing(kernel: &str, output: &str) -> Vec<String> {
    builds(&release(kernel), &parse_status(output))
        .into_iter()
        .map(|build| format!("{}/{} -k {}", build.module, build.version, build.kernel))
        .collect()
}

#[test]
fn dkms_status_lines_become_modules_per_kernel() {
    assert_eq!(
        shown(&parse_status(INSTALLED)),
        [
            "broadcom-wl/6.30.223.271 6.18.60-1-lts installed",
            "broadcom-wl/6.30.223.271 7.3.1-1-garuda installed",
            "v4l2loopback/0.15.4 6.18.60-1-lts installed",
            "v4l2loopback/0.15.4 7.3.1-1-garuda installed",
        ]
    );
}

#[test]
fn added_and_broken_lines_name_no_kernel() {
    assert_eq!(
        shown(&parse_status(ADDED))[2..],
        ["v4l2loopback/0.15.4 - added"]
    );
    assert_eq!(
        shown(&parse_status(BROKEN))[2..],
        ["v4l2loopback/0.15.4 - broken"]
    );
}

/// dkms 2.x wrote `name, version` where 3.x writes `name/version`, and a
/// built module says `built`. Both are edits of a real line.
#[test]
fn the_older_dkms_format_and_built_lines_are_read() {
    let comma_form = INSTALLED.replace("v4l2loopback/0.15.4", "v4l2loopback, 0.15.4");
    let built = ONE_KERNEL_MISSING.replace(
        "7.3.1-1-garuda, x86_64: installed (Original modules exist)",
        "7.3.1-1-garuda, x86_64: built",
    );

    assert_eq!(
        shown(&parse_status(&comma_form)),
        shown(&parse_status(INSTALLED))
    );
    assert_eq!(
        shown(&parse_status(&built))[2..],
        ["v4l2loopback/0.15.4 7.3.1-1-garuda built"]
    );
}

#[test]
fn empty_output_errors_and_odd_lines_give_no_modules() {
    let line = INSTALLED.lines().nth(2).unwrap();
    let unknown_state = line.replace(": installed", ": unknown");
    let no_version = "v4l2loopback: installed";

    assert_eq!(parse_status(""), []);
    assert_eq!(parse_status(BROKEN_STDERR), []);
    assert_eq!(parse_status(&unknown_state), []);
    assert_eq!(parse_status(no_version), []);
    assert_eq!(
        parse_status(&line.replace("v4l2loopback", "--force")),
        [],
        "a name that reads as an option"
    );
    assert_eq!(
        parse_status(&line.replace(LTS, &format!("../{LTS}"))),
        [],
        "a kernel that leaves the modules folder"
    );
}

#[test]
fn a_kernel_shows_each_module_once_with_its_state_there() {
    assert_eq!(
        states(LTS, ONE_KERNEL_MISSING),
        [
            ("broadcom-wl".to_owned(), ModuleState::Installed),
            ("v4l2loopback".to_owned(), ModuleState::Missing),
        ]
    );
    assert_eq!(
        states(GARUDA, ONE_KERNEL_MISSING),
        [
            ("broadcom-wl".to_owned(), ModuleState::Installed),
            ("v4l2loopback".to_owned(), ModuleState::Installed),
        ]
    );
}

#[test]
fn a_module_installed_for_the_kernel_is_not_missing() {
    assert_eq!(missing(GARUDA, INSTALLED), Vec::<String>::new());
    assert_eq!(missing(GARUDA, ONE_KERNEL_MISSING), Vec::<String>::new());
    assert_eq!(
        missing(LTS, ONE_KERNEL_MISSING),
        ["v4l2loopback/0.15.4 -k 6.18.60-1-lts"]
    );
}

#[test]
fn an_added_module_is_missing_on_every_kernel() {
    assert_eq!(
        missing(GARUDA, ADDED),
        ["v4l2loopback/0.15.4 -k 7.3.1-1-garuda"]
    );
    assert_eq!(
        missing(LTS, ADDED),
        ["v4l2loopback/0.15.4 -k 6.18.60-1-lts"]
    );
}

#[test]
fn a_module_without_its_source_is_broken_on_every_kernel() {
    for kernel in [GARUDA, LTS] {
        assert_eq!(
            states(kernel, BROKEN)[1],
            ("v4l2loopback".to_owned(), ModuleState::Broken)
        );
    }
}

/// Edit of `one-kernel-missing`: an older `v4l2loopback/0.15.3: added`
/// line before the rest.
#[test]
fn an_older_version_left_behind_does_not_count_as_missing() {
    let output = format!("v4l2loopback/0.15.3: added\n{ONE_KERNEL_MISSING}");

    assert_eq!(missing(GARUDA, &output), Vec::<String>::new());
    assert_eq!(
        missing(LTS, &output),
        ["v4l2loopback/0.15.4 -k 6.18.60-1-lts"]
    );
}
