use proptest::prelude::*;

use super::{InvalidMirrorlist, MAX_MIRRORLIST_BYTES, Mirrorlist};

const CHAOTIC: &str = "\
# Chaotic-AUR mirrorlist
# Global CDN
Server = https://cdn-mirror.chaotic.cx/$repo/$arch

# Germany
Server = https://de-mirror.chaotic.cx/$repo/$arch
";

#[test]
fn a_real_mirrorlist_is_accepted() {
    let mirrorlist = Mirrorlist::new(CHAOTIC).unwrap();

    assert_eq!(mirrorlist.as_str(), CHAOTIC);
    assert_eq!(mirrorlist.servers(), 2);
}

#[test]
fn a_mirrorlist_needs_a_server() {
    assert_eq!(
        Mirrorlist::new("# only comments\n\n").unwrap_err(),
        InvalidMirrorlist::NoServer
    );
    assert_eq!(
        Mirrorlist::new("").unwrap_err(),
        InvalidMirrorlist::NoServer
    );
}

#[test]
fn a_mirrorlist_has_a_size_limit() {
    let comment = format!("# {}\n", "x".repeat(MAX_MIRRORLIST_BYTES));
    let text = format!("Server = https://a.example/$repo\n{comment}");

    assert_eq!(
        Mirrorlist::new(text).unwrap_err(),
        InvalidMirrorlist::TooLarge {
            max: MAX_MIRRORLIST_BYTES
        }
    );
}

#[test]
fn only_comments_and_https_servers_are_allowed() {
    let rejected = [
        "Server = http://a.example/$repo",
        "Server=https://a.example/$repo",
        "Include = /etc/passwd",
        "[chaotic-aur]",
        "Server = https://a.example/$repo extra",
        "Server = https://a.example/\"$repo\"",
        "Server = https://a.example/'x'",
        "Server = https://a.example/`id`",
        "Server = https://a.example/$(id)",
        "Server = https://a.example/${HOME}",
        "Server = https://a.example",
        "Server = https://-bad.example/x",
        "Server = https://bad-.example/x",
        "Server = https://a..example/x",
        "Server = https:///x",
        "Server = https://a.example:8443/x",
        "Server = https://user@a.example/x",
        " Server = https://a.example/x",
        "Server = https://a.example/x\u{7}",
        "# bell \u{7}",
    ];

    for line in rejected {
        let text = format!("Server = https://ok.example/$repo/$arch\n{line}\n");
        assert_eq!(
            Mirrorlist::new(text).unwrap_err(),
            InvalidMirrorlist::InvalidLine { line: 2 },
            "{line:?}"
        );
    }
}

#[test]
fn hostnames_follow_the_dns_rules() {
    let long_label = "a".repeat(64);
    let long_host = ["a"; 128].join(".");

    for host in [long_label.as_str(), long_host.as_str()] {
        let text = format!("Server = https://{host}/x\n");
        assert_eq!(
            Mirrorlist::new(text).unwrap_err(),
            InvalidMirrorlist::InvalidLine { line: 1 },
            "{host}"
        );
    }

    Mirrorlist::new("Server = https://Mirror-1.Example.ORG/arch/$repo/os/$arch\n").unwrap();
}

#[test]
fn windows_line_endings_and_blank_lines_are_fine() {
    let mirrorlist =
        Mirrorlist::new("# a\r\n\r\n   \r\nServer = https://a.example/$repo\r\n").unwrap();

    assert_eq!(mirrorlist.servers(), 1);
}

#[test]
fn mirrorlists_are_strings_on_the_wire() {
    let mirrorlist = Mirrorlist::new(CHAOTIC).unwrap();
    let json = serde_json::to_string(&mirrorlist).unwrap();

    assert_eq!(json, serde_json::to_string(CHAOTIC).unwrap());
    assert_eq!(
        serde_json::from_str::<Mirrorlist>(&json).unwrap(),
        mirrorlist
    );
}

#[test]
fn an_invalid_mirrorlist_does_not_deserialize() {
    serde_json::from_str::<Mirrorlist>(r#""Include = /etc/shadow""#).unwrap_err();
}

proptest! {
    #[test]
    fn generated_server_lines_are_accepted(
        labels in prop::collection::vec("[a-z0-9]([a-z0-9-]{0,10}[a-z0-9])?", 1..3),
    ) {
        let text = labels
            .iter()
            .map(|label| format!("Server = https://{label}.example/$repo/$arch"))
            .collect::<Vec<_>>()
            .join("\n");
        let mirrorlist = Mirrorlist::new(format!("# mirrors\n{text}\n")).unwrap();

        prop_assert_eq!(mirrorlist.servers(), labels.len());
    }

    #[test]
    fn a_control_character_is_refused(line in "[ -~]{0,40}") {
        let text = format!("Server = https://ok.example/$repo/$arch\n{line}\u{7}\n");
        let reason = Mirrorlist::new(text).unwrap_err();

        prop_assert_eq!(reason, InvalidMirrorlist::InvalidLine { line: 2 });
    }

    #[test]
    fn generated_mirrorlists_keep_every_server(
        labels in prop::collection::vec("[a-zA-Z0-9]([a-zA-Z0-9-]{0,10}[a-zA-Z0-9])?", 1..3),
        extra in prop::collection::vec("[a-z]{1,8}", 0..2),
        servers in 1..4_usize,
    ) {
        let host = format!("{}.example", labels.join("."));
        let path = extra.iter().fold(String::from("/$repo/$arch"), |mut path, part| {
            path.push('/');
            path.push_str(part);
            path
        });
        let text = (0..servers)
            .map(|n| {
                if n % 2 == 0 {
                    format!("# mirror {n}\n")
                } else {
                    String::new()
                }
            })
            .chain(std::iter::repeat_n(
                format!("Server = https://{host}{path}"),
                servers,
            ))
            .collect::<Vec<_>>()
            .join("\n");
        let listed = format!("{text}\n");
        let mirrorlist = Mirrorlist::new(listed.clone()).unwrap();

        prop_assert_eq!(mirrorlist.servers(), servers);
        prop_assert_eq!(mirrorlist.as_str(), listed.as_str());
    }
}

#[test]
fn the_debug_form_names_no_server() {
    let mirrorlist = Mirrorlist::new("Server = https://a.example/$repo\n").unwrap();
    let debug = format!("{mirrorlist:?}");

    assert_eq!(debug, "Mirrorlist { servers: 1, bytes: 33 }");
}
