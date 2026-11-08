use toolbox_plan::KernelRelease;

use super::{DkmsState, ModuleState, builds, modules, parse_status};

fn release(name: &str) -> KernelRelease {
    KernelRelease::new(name).unwrap()
}

#[test]
fn dkms_status_lines_become_modules_per_kernel() {
    let entries = parse_status(
        "acpi_call/1.2.2, 6.1.1-a: installed\nvboxhost/7.0, 6.1.1-a: installed (original)",
    );

    let shown: Vec<(&str, &str, Option<&str>, DkmsState)> = entries
        .iter()
        .map(|entry| {
            (
                entry.module.as_str(),
                entry.version.as_str(),
                entry.kernel.as_ref().map(KernelRelease::as_str),
                entry.state,
            )
        })
        .collect();

    assert_eq!(
        shown,
        [
            ("acpi_call", "1.2.2", Some("6.1.1-a"), DkmsState::Installed),
            ("vboxhost", "7.0", Some("6.1.1-a"), DkmsState::Installed),
        ]
    );
}

#[test]
fn every_dkms_output_format_is_read() {
    let entries = parse_status(
        "nvidia/550.90.07, 6.14.0-arch1-1, x86_64: installed\n\
         nvidia, 470.74, 5.14.8-arch1-1, x86_64: installed\n\
         vboxhost/7.1.6_OSE, 6.12.9-1-lts, x86_64: installed (original_module exists) (WARNING! Diff between built and installed module!)\n\
         acpi_call/1.2.2, 6.14.0-arch1-1, x86_64: built\n\
         zfs/2.3.1: added\n\
         zfs, 2.2.0: added\n\
         nvidia/550.1, 6.2.2-z: broken\n",
    );

    let shown: Vec<String> = entries
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
        .collect();

    assert_eq!(
        shown,
        [
            "nvidia/550.90.07 6.14.0-arch1-1 installed",
            "nvidia/470.74 5.14.8-arch1-1 installed",
            "vboxhost/7.1.6_OSE 6.12.9-1-lts installed",
            "acpi_call/1.2.2 6.14.0-arch1-1 built",
            "zfs/2.3.1 - added",
            "zfs/2.2.0 - added",
            "nvidia/550.1 6.2.2-z broken",
        ]
    );
}

#[test]
fn empty_output_errors_and_odd_lines_give_no_modules() {
    assert_eq!(parse_status(""), []);
    assert_eq!(
        parse_status(
            "Error! Could not locate dkms.conf file.\n\
             dkms: command not found\n\
             nvidia/550.1, 6.2.2-z: unknown\n\
             --force/1, 6.2.2-z: installed\n\
             nvidia/550.1, ../6.2.2: installed\n\
             nvidia: installed\n"
        ),
        []
    );
}

#[test]
fn a_kernel_shows_each_module_once_with_its_state_there() {
    let entries = parse_status(
        "acpi_call/1.2.2, 6.14.0-4-cachyos, x86_64: installed\n\
         vboxhost/7.1.6_OSE, 6.12.9-1-cachyos, x86_64: broken\n\
         vboxhost/7.1.6_OSE, 6.14.0-4-cachyos, x86_64: installed\n\
         zfs/2.3.1, 6.12.9-1-cachyos, x86_64: built\n",
    );

    let states = |kernel: &str| -> Vec<(String, ModuleState)> {
        modules(&release(kernel), &entries)
            .into_iter()
            .map(|module| (module.module.to_string(), module.state))
            .collect()
    };

    assert_eq!(
        states("6.14.0-4-cachyos"),
        [
            ("acpi_call".to_owned(), ModuleState::Installed),
            ("vboxhost".to_owned(), ModuleState::Installed),
            ("zfs".to_owned(), ModuleState::Missing),
        ]
    );
    assert_eq!(
        states("6.12.9-1-cachyos"),
        [
            ("acpi_call".to_owned(), ModuleState::Missing),
            ("vboxhost".to_owned(), ModuleState::Broken),
            ("zfs".to_owned(), ModuleState::Built),
        ]
    );
}

/// `6.14.0-4-arch1-1` is the release of the modules folder, so only
/// `vboxhost` is missing.
#[test]
fn a_module_installed_for_the_kernel_is_not_missing() {
    let entries = parse_status("nvidia/550, 6.14.0-4-arch1-1: installed\nvboxhost/7.0: added\n");
    let missing: Vec<String> = builds(&release("6.14.0-4-arch1-1"), &entries)
        .into_iter()
        .map(|build| format!("{}/{} -k {}", build.module, build.version, build.kernel))
        .collect();

    assert_eq!(missing, ["vboxhost/7.0 -k 6.14.0-4-arch1-1"]);
}

#[test]
fn an_older_version_left_behind_does_not_count_as_missing() {
    let entries = parse_status(
        "nvidia/550.90.07: added\n\
         nvidia/560.35.03, 6.14.0-arch1-1, x86_64: installed\n\
         nvidia/560.35.03, 6.12.9-1-lts, x86_64: built\n",
    );

    assert_eq!(builds(&release("6.14.0-arch1-1"), &entries), []);

    let lts: Vec<String> = builds(&release("6.12.9-1-lts"), &entries)
        .into_iter()
        .map(|build| format!("{}/{}", build.module, build.version))
        .collect();

    assert_eq!(lts, ["nvidia/560.35.03"]);
}
