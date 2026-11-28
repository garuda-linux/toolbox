use super::choice::{BLUR_CAPABILITY, Offers, Path, decide, wayland_offers};

const EXTENSION: &str = "ext_background_effect_manager_v1";
const KDE: &str = "org_kde_kwin_blur_manager";
const COMPOSITOR: &str = "wl_compositor";

#[test]
fn the_standard_protocol_comes_before_kwin() {
    let offers = Offers {
        extension: true,
        kde: true,
        x11_kwin: true,
    };

    assert_eq!(decide(offers), Path::Extension);
}

#[test]
fn kwin_before_plasma_6_7_takes_the_kde_protocol() {
    let offers = Offers {
        kde: true,
        ..Offers::default()
    };

    assert_eq!(decide(offers), Path::KdeBlur);
}

#[test]
fn kwin_on_x11_takes_its_property() {
    let offers = Offers {
        x11_kwin: true,
        ..Offers::default()
    };

    assert_eq!(decide(offers), Path::X11Kwin);
    assert!(Path::X11Kwin.translucent());
}

#[test]
fn nothing_confirmed_is_opaque() {
    assert_eq!(decide(Offers::default()), Path::Opaque);
    assert!(!Path::Opaque.translucent());
}

#[test]
fn the_extension_without_the_blur_capability_is_not_taken() {
    let globals = [COMPOSITOR, EXTENSION];
    let offers = wayland_offers(&globals, Some(0));

    assert!(!offers.extension);
    assert_eq!(decide(offers), Path::Opaque);
}

#[test]
fn the_extension_is_taken_once_it_blurs() {
    let globals = [COMPOSITOR, EXTENSION];
    let offers = wayland_offers(&globals, Some(BLUR_CAPABILITY));

    assert!(offers.extension);
    assert_eq!(decide(offers), Path::Extension);
}

#[test]
fn no_capabilities_event_means_no_extension() {
    let globals = [COMPOSITOR, EXTENSION];

    assert!(!wayland_offers(&globals, None).extension);
}

#[test]
fn the_regions_need_the_compositor_global() {
    let without = [EXTENSION, KDE];
    let offers = wayland_offers(&without, Some(BLUR_CAPABILITY));

    assert!(!offers.extension);
    assert!(!offers.kde);
}

#[test]
fn kde_is_read_from_the_advertised_globals_among_unrelated_ones() {
    let globals = ["wl_shm", COMPOSITOR, "org_kde_plasma_shell", KDE];

    let offers = wayland_offers(&globals, None);

    assert!(offers.kde);
    assert!(!offers.extension);
    assert!(!offers.x11_kwin);
    assert_eq!(decide(offers), Path::KdeBlur);
}

#[test]
fn a_compositor_without_a_blur_protocol_stays_opaque() {
    let globals = [COMPOSITOR, "wl_shm"];
    let offers = wayland_offers(&globals, None);

    assert_eq!(offers, Offers::default());
    assert_eq!(decide(offers), Path::Opaque);
}

#[test]
fn the_paths_name_themselves_for_the_log() {
    assert_eq!(Path::Extension.to_string(), "ext-background-effect-v1");
    assert_eq!(Path::KdeBlur.to_string(), "org_kde_kwin_blur");
    assert_eq!(Path::X11Kwin.to_string(), "_KDE_NET_WM_BLUR_BEHIND_REGION");
    assert_eq!(Path::Opaque.to_string(), "opaque");
}
