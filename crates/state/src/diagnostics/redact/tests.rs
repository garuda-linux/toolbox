use super::{Kind, Known, redact};

/// `journalctl -e`, `dmesg --color=always` and `/var/log/pacman.log` of
/// the Garuda Lima VM. Its user's name is replaced by `user`, its disk
/// UUIDs by `x`.
const JOURNAL: &str = include_str!("../fixtures/installed/journal.txt");
const DMESG: &str = include_str!("../fixtures/installed/dmesg.txt");
const PACMAN_LOG: &str = include_str!("../fixtures/installed/pacman.log");

const HOST: &str = "lima-garuda-toolbox-it";
const MAC: &str = "52:55:55:1e:44:5b";

fn lima() -> Known {
    Known {
        user: Some("user".to_owned()),
        host: Some(HOST.to_owned()),
        home: Some("/home/user.guest".to_owned()),
        serials: Vec::new(),
    }
}

#[test]
fn the_vm_s_names_and_hardware_address_leave_the_journal() {
    let redacted = redact(JOURNAL, &lima());

    for gone in [HOST, MAC, "/home/user.guest", "139.162.152.20"] {
        assert!(JOURNAL.contains(gone), "{gone} not in the capture");
        assert!(!redacted.text().contains(gone), "{gone} stayed");
    }

    assert!(redacted.text().contains("<home>/.profile"));
    assert!(redacted.text().contains("<mac-1>"));
    assert_eq!(redacted.count(Kind::Host), JOURNAL.matches(HOST).count());
}

/// The VM's link-local address carries its MAC (EUI-64), the private one
/// says nothing about who sent the report.
#[test]
fn link_local_goes_and_private_addresses_stay() {
    let redacted = redact(JOURNAL, &lima());

    assert!(JOURNAL.contains("fe80::5055:55ff:fe1e:445b"));
    assert!(!redacted.text().contains("fe80::5055:55ff:fe1e:445b"));
    assert!(redacted.text().contains("192.168.5.15"));
    assert!(
        redact(DMESG, &lima())
            .text()
            .contains("hostname=192.168.5.2")
    );
}

#[test]
fn versions_and_reverse_zones_are_no_addresses() {
    assert_eq!(redact(PACMAN_LOG, &lima()).count(Kind::Address), 0);
    assert!(
        redact(JOURNAL, &lima())
            .text()
            .contains("170.0.0.192.in-addr.arpa")
    );
}

/// Changed: lines with an IPv6 address, a UUID, a serial and a second
/// public address inserted after the capture's first line.
#[test]
fn inserted_values_get_stable_numbered_placeholders() {
    let (first, rest) = DMESG.split_once('\n').unwrap();
    let inserted = format!(
        "{first}\nfrom 2a01:4f8:c0c:1234::1 and 2001:db8::1 and ::1\n\
         disk 0f1e2d3c-4b5a-6978-8796-a5b4c3d2e1f0 serial PF3XYZ12\n\
         again 2a01:4f8:c0c:1234::1 then 8.8.4.4\n{rest}"
    );

    let known = Known {
        serials: vec!["PF3XYZ12".to_owned()],
        ..lima()
    };

    let redacted = redact(&inserted, &known);

    assert!(
        redacted
            .text()
            .contains("from <ip-1> and 2001:db8::1 and ::1")
    );
    assert!(redacted.text().contains("disk <uuid-1> serial <serial-1>"));
    assert!(redacted.text().contains("again <ip-1> then <ip-2>"));
    assert_eq!(redacted.count(Kind::Address), 3);
    assert_eq!(redacted.count(Kind::Serial), 1);
}

#[test]
fn a_name_inside_a_word_stays() {
    let known = Known {
        user: Some("nico".to_owned()),
        ..Known::default()
    };

    let redacted = redact("unicode for nico", &known);

    assert_eq!(redacted.text(), "unicode for <user>");
    assert_eq!(redacted.total(), 1);
}
