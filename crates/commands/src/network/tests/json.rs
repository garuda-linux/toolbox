use toolbox_plan::{KernelModule, Toggle};

use super::super::StatusView;
use crate::network::DriverLine;

/// The module is a plain name in JSON.
#[test]
fn a_driver_line_as_json_keeps_its_module_as_a_string() {
    let line = DriverLine {
        module: KernelModule::new("iwlwifi").unwrap(),
        category: "wireless",
        loaded: true,
        blacklisted: false,
    };

    assert_eq!(
        serde_json::to_string(&line).unwrap(),
        r#"{"id":"iwlwifi","category":"wireless","loaded":true,"blacklisted":false}"#
    );
}

/// Scripts read the radios as `true`, `false` or `null` under `--json`.
#[test]
fn the_radios_keep_their_boolean_or_null_json() {
    let view = StatusView {
        wifi: Some(Toggle::On),
        bluetooth: None,
        ..StatusView::default()
    };

    assert_eq!(
        serde_json::to_string(&view).unwrap(),
        r#"{"gateway":null,"interface":null,"interfaces":[],"wifi":true,"bluetooth":null,"flight_mode":false,"unread":[]}"#
    );
}
