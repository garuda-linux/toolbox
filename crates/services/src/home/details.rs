//! The system details About copies for a forum post or a bug report. Like
//! the Diagnostics reports they stay in English, so the people who help
//! can read them, and they leave out the user's name.

#[cfg(test)]
mod tests;

use std::fmt::Write as _;

use toolbox_system::{Facts, Host};

/// One line per fact that was read, the app and its version first.
pub fn system_details(version: &str, host: Option<&Host>, facts: Option<&Facts>) -> String {
    let mut text = format!("Garuda Toolbox {version}\n");

    if let Some(host) = host {
        if let Some(edition) = &host.edition {
            _ = writeln!(text, "Edition: {edition}");
        }

        if !host.desktops.is_empty() {
            _ = writeln!(text, "Desktop: {}", host.desktops.join(", "));
        }

        _ = writeln!(
            text,
            "Live system: {}",
            if host.live_iso { "yes" } else { "no" }
        );
    }

    let Some(facts) = facts else {
        return text;
    };

    if let Ok(kernel) = &facts.kernel {
        _ = writeln!(text, "Kernel: {kernel}");
    }

    if let Ok(count) = &facts.packages {
        let foreign = count
            .foreign
            .filter(|foreign| *foreign > 0)
            .map(|foreign| format!(" ({foreign} foreign)"))
            .unwrap_or_default();

        _ = writeln!(text, "Packages: {}{foreign}", count.installed);
    }

    text
}
