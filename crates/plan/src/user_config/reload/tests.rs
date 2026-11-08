use super::ConfigReload;

#[test]
fn kwin_is_told_by_its_reload_signal() {
    assert_eq!(
        ConfigReload::Kwin.command().to_string(),
        "dbus-send --session --type=signal /KWin org.kde.KWin.reloadConfig"
    );
}

#[test]
fn ids_round_trip() {
    for reload in ConfigReload::ALL {
        assert_eq!(ConfigReload::from_id(reload.as_str()), Some(reload));
    }

    assert_eq!(ConfigReload::from_id("plasmashell"), None);
}
