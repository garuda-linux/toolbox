use toolbox_plan::UserName;
use toolbox_system::{DiskSpace, Facts, Host, PackageCount, ProbeError};

use super::system_details;

fn host() -> Host {
    Host {
        user: Some(UserName::new("ada").unwrap()),
        live_iso: false,
        desktops: vec!["KDE".to_owned()],
        edition: Some("dr460nized".to_owned()),
        reboot_pending: false,
    }
}

fn facts() -> Facts {
    Facts {
        packages: Ok(PackageCount {
            installed: 1_462,
            foreign: Some(12),
        }),
        kernel: Ok("6.17.7-zen1-1-zen".to_owned()),
        booted: Err(ProbeError::Unavailable),
        disk: Ok(DiskSpace {
            total: 500,
            free: 200,
        }),
        package_cache: Ok(0),
        orphans: Ok(Vec::new()),
        failed_units: Ok(Vec::new()),
    }
}

#[test]
fn the_details_name_the_app_the_system_and_its_packages() {
    let text = system_details("2.0.1", Some(&host()), Some(&facts()));

    assert_eq!(
        text,
        "Garuda Toolbox 2.0.1\n\
         Edition: dr460nized\n\
         Desktop: KDE\n\
         Live system: no\n\
         Kernel: 6.17.7-zen1-1-zen\n\
         Packages: 1462 (12 foreign)\n"
    );
}

#[test]
fn the_details_never_name_the_user() {
    let text = system_details("2.0.1", Some(&host()), Some(&facts()));

    assert!(!text.contains("ada"), "{text}");
}

#[test]
fn what_was_not_read_is_left_out() {
    let mut read = facts();
    read.kernel = Err(ProbeError::Unavailable);
    read.packages = Err(ProbeError::Unavailable);

    assert_eq!(
        system_details("2.0.1", None, Some(&read)),
        "Garuda Toolbox 2.0.1\n"
    );
    assert_eq!(
        system_details("2.0.1", None, None),
        "Garuda Toolbox 2.0.1\n"
    );
}
