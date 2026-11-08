use super::Family;

#[test]
fn kde_and_lxqt_read_kdeglobals() {
    assert_eq!(Family::from_current_desktop("KDE"), Family::Kde);
    assert_eq!(Family::from_current_desktop("LXQt"), Family::Kde);
}

#[test]
fn gtk_desktops_read_gtk_files() {
    for desktop in [
        "GNOME",
        "ubuntu:GNOME",
        "X-Cinnamon",
        "XFCE",
        "Budgie:GNOME",
    ] {
        assert_eq!(
            Family::from_current_desktop(desktop),
            Family::Gtk,
            "{desktop}"
        );
    }
}

#[test]
fn names_match_without_case() {
    assert_eq!(Family::from_current_desktop("kde"), Family::Kde);
    assert_eq!(Family::from_current_desktop("gnome"), Family::Gtk);
}

#[test]
fn other_desktops_use_only_the_portal() {
    for desktop in ["Hyprland", "sway", "", "niri"] {
        assert_eq!(
            Family::from_current_desktop(desktop),
            Family::Other,
            "{desktop}"
        );
    }
}
