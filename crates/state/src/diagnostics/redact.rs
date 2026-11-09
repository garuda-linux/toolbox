//! A report without what names the person or the machine: their user and
//! host name, the home folder, hardware addresses, public IP addresses,
//! UUIDs and serials. One value keeps one stand-in through the whole
//! report, so the helpers who read it still see that two lines speak of
//! one disk.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::LazyLock;

use regex::Regex;

static UUID: LazyLock<Regex> = LazyLock::new(|| {
    pattern(r"\b[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\b")
});

static MAC: LazyLock<Regex> =
    LazyLock::new(|| pattern(r"\b[0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2}){5}\b"));

static IPV6: LazyLock<Regex> =
    LazyLock::new(|| pattern("[0-9A-Fa-f]{0,4}(?::[0-9A-Fa-f]{0,4}){2,7}"));

static IPV4: LazyLock<Regex> = LazyLock::new(|| pattern(r"\b\d{1,3}(?:\.\d{1,3}){3}\b"));

/// The values only the system knows, which no pattern finds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Known {
    pub user: Option<String>,
    pub host: Option<String>,
    pub home: Option<String>,
    /// From `/sys/class/dmi/id`, empty ones left out.
    pub serials: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Home,
    User,
    Host,
    Serial,
    Uuid,
    Mac,
    Address,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Redacted {
    text: String,
    replaced: BTreeMap<Kind, usize>,
}

impl Redacted {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn into_text(self) -> String {
        self.text
    }

    /// How many places held a value of `kind`.
    pub fn count(&self, kind: Kind) -> usize {
        self.replaced.get(&kind).copied().unwrap_or(0)
    }

    pub fn total(&self) -> usize {
        self.replaced.values().sum()
    }

    fn literal(&mut self, value: &str, placeholder: &str, kind: Kind) {
        let found = self.text.matches(value).count();

        if found > 0 {
            self.text = self.text.replace(value, placeholder);
            *self.replaced.entry(kind).or_default() += found;
        }
    }

    /// `value` as a whole word only, so a short name inside another word
    /// stays.
    fn word(&mut self, value: &str, placeholder: &str, kind: Kind) {
        let source = format!(r"\b{}\b", regex::escape(value));
        let Ok(word) = Regex::new(&source) else {
            return;
        };

        let found = word.find_iter(&self.text).count();

        if found > 0 {
            self.text = word.replace_all(&self.text, placeholder).into_owned();
            *self.replaced.entry(kind).or_default() += found;
        }
    }

    /// Each distinct value of the spans `find` gives becomes
    /// `<name-N>`, numbered in the order the values first appear.
    fn numbered(&mut self, kind: Kind, name: &str, find: impl Fn(&str) -> Vec<(usize, usize)>) {
        let mut spans = find(&self.text);

        if spans.is_empty() {
            return;
        }

        spans.sort_unstable();
        spans.dedup_by(|later, earlier| later.0 < earlier.1);

        let mut numbers: BTreeMap<&str, usize> = BTreeMap::new();
        let mut text = String::with_capacity(self.text.len());
        let mut last = 0;

        for &(start, end) in &spans {
            let value = self.text.get(start..end).unwrap_or_default();
            let next = numbers.len() + 1;
            let number = *numbers.entry(value).or_insert(next);

            text.push_str(self.text.get(last..start).unwrap_or_default());
            write!(text, "<{name}-{number}>").ok();
            last = end;
        }

        text.push_str(self.text.get(last..).unwrap_or_default());
        *self.replaced.entry(kind).or_default() += spans.len();
        self.text = text;
    }
}

pub fn redact(text: &str, known: &Known) -> Redacted {
    let mut redacted = Redacted {
        text: text.to_owned(),
        replaced: BTreeMap::new(),
    };

    if let Some(home) = known.home.as_deref().filter(|home| !home.is_empty()) {
        redacted.literal(home, "<home>", Kind::Home);
    }

    for (value, placeholder, kind) in [
        (&known.user, "<user>", Kind::User),
        (&known.host, "<host>", Kind::Host),
    ] {
        if let Some(value) = value.as_deref().filter(|value| !value.is_empty()) {
            redacted.word(value, placeholder, kind);
        }
    }

    let serials: Vec<&str> = known
        .serials
        .iter()
        .map(|serial| serial.trim())
        .filter(|serial| !serial.is_empty())
        .collect();

    redacted.numbered(Kind::Serial, "serial", |haystack| {
        serials
            .iter()
            .flat_map(|serial| {
                haystack
                    .match_indices(serial)
                    .map(|(at, _)| (at, at + serial.len()))
            })
            .collect()
    });
    redacted.numbered(Kind::Uuid, "uuid", |haystack| {
        spans(&UUID, haystack, |_| true)
    });
    redacted.numbered(Kind::Mac, "mac", |haystack| {
        spans(&MAC, haystack, |mac| !is_blank_mac(mac))
    });
    redacted.numbered(Kind::Address, "ip", addresses);

    redacted
}

#[expect(
    clippy::expect_used,
    reason = "a fixed pattern, which the tests compile"
)]
fn pattern(source: &str) -> Regex {
    Regex::new(source).expect("the pattern compiles")
}

fn addresses(haystack: &str) -> Vec<(usize, usize)> {
    let mut found = spans(&IPV6, haystack, |candidate| {
        candidate
            .parse::<Ipv6Addr>()
            .is_ok_and(|address| !keeps_v6(address))
    });

    found.extend(spans(&IPV4, haystack, |candidate| {
        candidate
            .parse::<Ipv4Addr>()
            .is_ok_and(|address| !keeps_v4(address))
    }));
    found.retain(|&(start, end)| stands_alone(haystack, start, end));
    found
}

fn spans(pattern: &Regex, haystack: &str, keep: impl Fn(&str) -> bool) -> Vec<(usize, usize)> {
    pattern
        .find_iter(haystack)
        .filter(|found| keep(found.as_str()))
        .map(|found| (found.start(), found.end()))
        .collect()
}

/// A dotted number inside a version (`3.3.41.1-1`) or a reverse zone
/// (`170.0.0.192.in-addr.arpa`) is no address.
fn stands_alone(haystack: &str, start: usize, end: usize) -> bool {
    let before = haystack
        .get(..start)
        .and_then(|head| head.chars().next_back());

    let after = haystack.get(end..).and_then(|tail| tail.chars().next());
    let joined = |character: Option<char>| {
        character.is_some_and(|character| character.is_alphanumeric() || ".-_".contains(character))
    };

    !joined(before) && !joined(after)
}

const fn is_blank_mac(mac: &str) -> bool {
    matches!(
        mac.as_bytes(),
        b"00:00:00:00:00:00" | b"ff:ff:ff:ff:ff:ff" | b"FF:FF:FF:FF:FF:FF"
    )
}

/// Private, loopback and documentation addresses name no one.
const fn keeps_v4(address: Ipv4Addr) -> bool {
    address.is_private()
        || address.is_loopback()
        || address.is_unspecified()
        || address.is_documentation()
        || address.is_broadcast()
        || address.is_multicast()
}

/// Unique local, loopback and documentation addresses name no one. A
/// link-local one goes: it is often made of the MAC.
const fn keeps_v6(address: Ipv6Addr) -> bool {
    let [first, second, ..] = address.segments();

    address.is_loopback()
        || address.is_unspecified()
        || address.is_multicast()
        || first & 0xfe00 == 0xfc00
        || (first == 0x2001 && second == 0x0db8)
}
