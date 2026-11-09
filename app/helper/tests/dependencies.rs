//! The helper runs as root, so every crate it links is code with root. Its
//! graph stays free of a runtime, a GUI, D-Bus, HTTP and `parking_lot`.

use std::error::Error;
use std::path::Path;
use std::process::Command;

const BANNED: [&str; 5] = ["parking_lot", "tokio", "egui", "zbus", "ureq"];

/// Proof that the listing is the helper's, not an empty or broken one.
const EXPECTED: [&str; 2] = ["toolbox-exec", "toolbox-plan"];

type TestResult = Result<(), Box<dyn Error>>;

/// Every crate the helper links, one name each.
fn normal_dependencies() -> Result<Vec<String>, Box<dyn Error>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

    let output = Command::new(cargo)
        .args(["tree", "--frozen", "--edges", "normal", "--prefix", "none"])
        .args(["--target", "all"])
        .args(["--format", "{p}", "-p", "toolbox-helper", "--manifest-path"])
        .arg(manifest)
        .output()?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }

    Ok(String::from_utf8(output.stdout)?
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect())
}

#[test]
fn the_helper_links_no_runtime_gui_dbus_http_or_parking_lot() -> TestResult {
    let names = normal_dependencies()?;

    for expected in EXPECTED {
        assert!(
            names.iter().any(|name| name == expected),
            "{expected} missing from {names:?}"
        );
    }

    let banned: Vec<&String> = names
        .iter()
        .filter(|name| BANNED.contains(&name.as_str()))
        .collect();

    assert!(banned.is_empty(), "the helper links {banned:?}");
    Ok(())
}
