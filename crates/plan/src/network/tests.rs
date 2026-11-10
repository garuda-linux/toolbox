use std::collections::BTreeSet;

use super::{
    HopLimit, PingCount, PingTimeout, blacklist_entry, blacklisted_modules, set_blacklisted,
};
use crate::{HostName, KernelModule, OutOfRange, Reason, Toggle};

/// `/usr/lib/modprobe.d/systemd.conf` of the Lima image: comments and
/// `options` lines, no blacklist.
const SYSTEMD_CONF: &str = crate::testing::network::modules::ROOT_USR_LIB_MODPROBE_D_SYSTEMD_CONF;

fn module(name: &str) -> KernelModule {
    KernelModule::new(name).unwrap()
}

fn modules(names: &[&str]) -> BTreeSet<KernelModule> {
    names.iter().map(|name| module(name)).collect()
}

#[test]
fn hosts_are_dns_names_and_addresses() {
    for host in [
        "garudalinux.org",
        "localhost",
        "192.168.1.1",
        "2a01:4f8::1",
        "::1",
        "xn--mnchen-3ya.de",
    ] {
        assert_eq!(HostName::new(host).unwrap().as_str(), host);
    }
}

#[test]
fn a_host_can_never_be_an_option_or_two_words() {
    assert_eq!(
        HostName::new("-f").unwrap_err().reason(),
        Reason::InvalidStart('-')
    );
    assert_eq!(
        HostName::new("a.org -f").unwrap_err().reason(),
        Reason::InvalidCharacter(' ')
    );
    assert_eq!(
        HostName::new("a.org;reboot").unwrap_err().reason(),
        Reason::InvalidCharacter(';')
    );
    assert_eq!(
        HostName::new("$(id)").unwrap_err().reason(),
        Reason::InvalidStart('$')
    );
    assert_eq!(HostName::new("").unwrap_err().reason(), Reason::Empty);
    assert_eq!(
        HostName::new("a".repeat(254)).unwrap_err().reason(),
        Reason::TooLong { max: 253 }
    );
}

#[test]
fn numbers_stay_inside_their_range() {
    assert_eq!(PingCount::new(4).unwrap().get(), 4);
    assert_eq!(PingCount::new(100).unwrap().to_string(), "100");
    assert_eq!(
        PingCount::new(0).unwrap_err(),
        OutOfRange {
            value: 0,
            min: 1,
            max: 100
        }
    );
    PingCount::new(101).unwrap_err();
    PingCount::new(-1).unwrap_err();
    PingTimeout::new(61).unwrap_err();
    assert_eq!(HopLimit::new(255).unwrap().get(), 255);
    HopLimit::new(256).unwrap_err();
    HopLimit::new(i64::MAX).unwrap_err();
}

#[test]
fn a_number_from_the_wire_is_checked_again() {
    assert_eq!(
        serde_json::from_str::<PingCount>("4").unwrap(),
        PingCount::new(4).unwrap()
    );
    serde_json::from_str::<PingCount>("0").unwrap_err();
    serde_json::from_str::<PingCount>("4000").unwrap_err();
    serde_json::from_str::<PingCount>("\"4\"").unwrap_err();
    assert_eq!(
        serde_json::to_string(&HopLimit::new(30).unwrap()).unwrap(),
        "30"
    );
}

#[test]
fn only_plain_blacklist_lines_count() {
    assert_eq!(blacklist_entry("blacklist b43"), Some(module("b43")));
    assert_eq!(
        blacklist_entry("  blacklist\tbrcmsmac  # by broadcom-wl"),
        Some(module("brcmsmac"))
    );
    assert_eq!(blacklist_entry("# blacklist b43"), None);
    assert_eq!(blacklist_entry("install b43 /bin/false"), None);
    assert_eq!(blacklist_entry("blacklist"), None);
    assert_eq!(blacklist_entry("blacklist b43 ssb"), None);
    assert_eq!(blacklist_entry("blacklist ../evil"), None);
    assert_eq!(blacklist_entry("blacklisted b43"), None);
}

#[test]
fn a_dash_and_an_underscore_name_the_same_module() {
    assert_eq!(module("mt76-usb"), module("mt76_usb"));
    assert_eq!(
        blacklist_entry("blacklist mt76-usb"),
        Some(module("mt76_usb"))
    );
    assert_eq!(
        blacklisted_modules("blacklist mt76-usb\nblacklist mt76_usb\nblacklist wl\n"),
        modules(&["mt76_usb", "wl"])
    );
}

#[test]
fn the_first_entry_starts_the_file_with_its_origin() {
    assert_eq!(
        set_blacklisted("", &module("r8169"), Toggle::On),
        "# Written by Garuda Toolbox (Network, Drivers). Edit it there.\nblacklist r8169\n"
    );
}

#[test]
fn a_real_modprobe_file_without_blacklist_lines_lists_nothing() {
    assert_eq!(blacklisted_modules(SYSTEMD_CONF), BTreeSet::new());
    assert!(
        SYSTEMD_CONF
            .lines()
            .all(|line| blacklist_entry(line).is_none())
    );
}

#[test]
fn an_entry_is_appended_and_every_other_line_stays() {
    assert_eq!(
        set_blacklisted(SYSTEMD_CONF, &module("mt76-usb"), Toggle::On),
        format!("{SYSTEMD_CONF}blacklist mt76_usb\n")
    );

    // Edit: the real file with a blacklist line and no last line break.
    let before = format!("{SYSTEMD_CONF}blacklist wl");

    assert_eq!(
        set_blacklisted(&before, &module("mt76-usb"), Toggle::On),
        format!("{before}\nblacklist mt76_usb\n")
    );
}

#[test]
fn a_listed_module_is_not_listed_twice() {
    let before = "blacklist mt76-usb\n";

    assert_eq!(
        set_blacklisted(before, &module("mt76_usb"), Toggle::On),
        before
    );
}

#[test]
fn taking_a_module_off_removes_each_of_its_lines_only() {
    let before = "# mine\nblacklist wl\nblacklist b43\r\nblacklist wl # twice\noptions wl x=1\n";

    assert_eq!(
        set_blacklisted(before, &module("wl"), Toggle::Off),
        "# mine\nblacklist b43\r\noptions wl x=1\n"
    );
    assert_eq!(set_blacklisted(before, &module("ssb"), Toggle::Off), before);
}
