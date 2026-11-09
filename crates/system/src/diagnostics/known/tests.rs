use std::fs;
use std::path::Path;

use toolbox_plan::UserName;

use super::{KnownRoots, known};

const HOSTNAME: &str = include_str!("../fixtures/installed/proc/hostname");
const PRODUCT_SERIAL: &str = include_str!("../fixtures/installed/dmi/product_serial");
const CHASSIS_SERIAL: &str = include_str!("../fixtures/installed/dmi/chassis_serial");
const CHASSIS_ASSET_TAG: &str = include_str!("../fixtures/installed/dmi/chassis_asset_tag");

fn machine(root: &Path, product_serial: &str) -> KnownRoots {
    let dmi = root.join("dmi");
    fs::create_dir_all(&dmi).expect("the dmi folder is made");
    fs::write(dmi.join("product_serial"), product_serial).expect("the serial is written");
    fs::write(dmi.join("chassis_serial"), CHASSIS_SERIAL).expect("the serial is written");
    fs::write(dmi.join("chassis_asset_tag"), CHASSIS_ASSET_TAG).expect("the tag is written");
    fs::write(root.join("hostname"), HOSTNAME).expect("the host name is written");

    KnownRoots {
        dmi,
        hostname: root.join("hostname"),
    }
}

#[test]
fn the_vm_names_its_host_and_has_no_serial() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let roots = machine(folder.path(), PRODUCT_SERIAL);
    let user = UserName::new("user").expect("a valid name");

    let known = known(&roots, Some(&user), Some(Path::new("/home/user")));

    assert_eq!(known.host.as_deref(), Some("lima-garuda-toolbox-it"));
    assert_eq!(known.user.as_deref(), Some("user"));
    assert_eq!(known.home.as_deref(), Some("/home/user"));
    assert!(known.serials.is_empty(), "{:?}", known.serials);
}

#[test]
fn a_filled_in_serial_is_known() {
    let folder = tempfile::tempdir().expect("a temporary folder");

    // Changed: the VM's empty product serial filled in, as real hardware has it.
    let roots = machine(folder.path(), "PF2ABCDE\n");

    let known = known(&roots, None, None);

    assert_eq!(known.serials, ["PF2ABCDE"]);
}

#[test]
fn placeholder_serials_and_missing_files_are_left_out() {
    let folder = tempfile::tempdir().expect("a temporary folder");

    // Changed: the product serial is a vendor's filler text.
    let mut roots = machine(folder.path(), "To Be Filled By O.E.M.\n");
    roots.hostname = folder.path().join("missing");

    let known = known(&roots, None, None);

    assert!(known.serials.is_empty(), "{:?}", known.serials);
    assert_eq!(known.host, None);
}
