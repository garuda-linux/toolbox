use super::super::Sizes;

/// Kirigami's `preferredHeight` for the desktop global toolbar is
/// `40 + 6`, where 40 comes from Breeze and 6 is `ToolBar_ItemMargin`
/// (`PageRowGlobalToolBarStyleGroup.qml`, `breezemetrics.h:110`).
#[test]
fn the_toolbar_row_is_kirigamis_global_toolbar_height() {
    assert!((Sizes::DEFAULT.toolbar_height - (40.0 + 6.0)).abs() < 0.001);
}
