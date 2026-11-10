use super::{local, sample};
use crate::Snapshot;
use crate::features::kernels::{
    KernelReleases, LinuxPackages, RunningKernel, kernels, kernels_of, running,
};
use crate::test_support::package;

#[test]
fn the_snapshot_gives_the_kernels_and_the_one_that_runs() {
    let mut snapshot = Snapshot::default();
    assert_eq!(kernels_of(&snapshot), None);

    snapshot.insert(LinuxPackages {
        sync: Vec::new(),
        local: vec![local("linux", "1-1"), local("linux-headers", "1-1")],
    });
    snapshot.insert(RunningKernel {
        release: toolbox_plan::KernelRelease::new("6.14.0-arch1-1").unwrap(),
        package: None,
    });
    let (listed, booted) = kernels_of(&snapshot).unwrap();

    assert_eq!(listed.len(), 1);
    assert_eq!(booted, Some(package("linux")));
}

#[test]
fn the_modules_folder_names_the_running_kernel() {
    let (kernels, _) = sample();
    let booted = RunningKernel {
        release: toolbox_plan::KernelRelease::new("6.12.9-1-lts").unwrap(),
        package: Some(package("linux-lts")),
    };

    assert_eq!(
        running(&booted, &kernels).map(|kernel| kernel.name.as_str()),
        Some("linux-lts")
    );
}

#[test]
fn without_its_folder_the_release_ending_names_the_running_kernel() {
    let packages = LinuxPackages {
        sync: Vec::new(),
        local: [
            "linux",
            "linux-lts",
            "linux-zen",
            "linux-hardened",
            "linux-cachyos",
            "linux-cachyos-lts",
        ]
        .into_iter()
        .flat_map(|name| [local(name, "1-1"), local(&format!("{name}-headers"), "1-1")])
        .collect(),
    };

    let kernels = kernels(&packages, &KernelReleases::default());
    let by_release = |release: &str| {
        let booted = RunningKernel {
            release: toolbox_plan::KernelRelease::new(release).unwrap(),
            package: None,
        };

        running(&booted, &kernels).map(|kernel| kernel.name.to_string())
    };

    assert_eq!(by_release("6.14.0-arch1-1").as_deref(), Some("linux"));
    assert_eq!(by_release("6.12.9-1-lts").as_deref(), Some("linux-lts"));
    assert_eq!(
        by_release("6.14.0-zen1-1-zen").as_deref(),
        Some("linux-zen")
    );
    assert_eq!(
        by_release("6.14.0-hardened1-1-hardened").as_deref(),
        Some("linux-hardened")
    );
    assert_eq!(
        by_release("6.14.0-4-cachyos").as_deref(),
        Some("linux-cachyos")
    );
    assert_eq!(
        by_release("6.12.9-1-cachyos-lts").as_deref(),
        Some("linux-cachyos-lts")
    );
    assert_eq!(by_release("6.14.0-unknown1-1-foo"), None);
}
