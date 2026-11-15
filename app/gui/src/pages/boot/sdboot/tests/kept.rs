use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::{LOADER_CONF, image_menu, shown};

/// Why a parameter stays reaches screen readers, not only the pointer.
#[test]
fn a_kept_parameter_tells_screen_readers_why_it_stays() {
    let harness = shown(image_menu(LOADER_CONF));
    let why = "rw stays: without it the system finds no root file system to start.";

    let kept = harness.get_by_label("rw");
    assert_eq!(kept.accesskit_node().description().as_deref(), Some(why));
}
