//! The probe and the helper share these pure text functions, so both read a blacklist line the same way.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use crate::lines::push_line;
use crate::{Bounded, KernelModule};

/// The only modprobe file the app writes, relative to `/`. Files of
/// packages and of the administrator are read, never changed.
pub const MODPROBE_BLACKLIST: &str = "etc/modprobe.d/garuda-toolbox-blacklist.conf";

pub const DEFAULT_TEST_HOST: &str = "garudalinux.org";
/// The echo requests, the seconds per answer and the routers the tests
/// start with, on the page and on the command line.
pub const DEFAULT_PINGS: u32 = 4;
pub const DEFAULT_PING_TIMEOUT: u32 = 5;
pub const DEFAULT_HOPS: u32 = 30;

const KEYWORD: &str = "blacklist";

const HEADER: &str = "# Written by Garuda Toolbox (Network, Drivers). Edit it there.\n";

pub type PingCount = Bounded<1, 100>;
/// How many seconds ping waits for each answer.
pub type PingTimeout = Bounded<1, 60>;
/// The most routers traceroute walks through. 255 is the largest time to
/// live an IP packet can carry.
pub type HopLimit = Bounded<1, 255>;

/// The module a line of a modprobe file blacklists, in its `_` spelling.
/// `None` for comments, other commands and names that are no module name.
pub fn blacklist_entry(line: &str) -> Option<KernelModule> {
    let mut words = line.split('#').next()?.split_whitespace();

    if words.next()? != KEYWORD {
        return None;
    }

    let module = KernelModule::new(words.next()?).ok()?;

    words.next().is_none().then_some(module)
}

pub fn blacklisted_modules(config: &str) -> BTreeSet<KernelModule> {
    config.lines().filter_map(blacklist_entry).collect()
}

/// `config` with `module` blacklisted or not. Every other line stays. A
/// new file starts with a line that says who wrote it.
pub fn set_blacklisted(config: &str, module: &KernelModule, on: bool) -> String {
    let listed = blacklisted_modules(config).contains(module);

    if on == listed {
        return config.to_owned();
    }

    if !on {
        return config
            .split_inclusive('\n')
            .filter(|line| blacklist_entry(line).as_ref() != Some(module))
            .collect();
    }

    let mut result = if config.is_empty() {
        HEADER.to_owned()
    } else {
        config.to_owned()
    };

    push_line(&mut result, &format!("{KEYWORD} {module}"));
    result
}
