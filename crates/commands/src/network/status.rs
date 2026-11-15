//! `network show`: the route, the radios, the interface cards and the reads
//! that failed, as text or JSON.

use std::io::Write as _;

use toolbox_cli::Exit;
use toolbox_cli::args::Read;
use toolbox_system::NetworkSource;

use super::interfaces::{InterfaceView, interface_facts};
use super::{radio_listing, view};
use crate::json;
use crate::out::{Out, failure};

pub fn status(
    read: Read,
    source: &dyn NetworkSource,
    interfaces: impl FnOnce() -> Vec<InterfaceView>,
    out: &mut Out<'_>,
) -> Exit {
    let view = crate::spinner::while_reading(read.json || out.quiet(), || {
        let mut view = view(source);
        view.interfaces = interfaces();
        view
    });

    if read.json {
        write!(out, "{}", json(&view)).ok();
    } else {
        let paint = out.paint();
        let absent = if view.unread.contains(&"router") {
            "not read"
        } else {
            "none"
        };

        let facts = paint.facts(
            "Network",
            &[
                ("router", view.gateway.as_deref().unwrap_or(absent)),
                ("interface", view.interface.as_deref().unwrap_or(absent)),
            ],
        );

        let radios = radio_listing(&view);
        let summary = radios.summary.as_deref();
        let list = paint.list(radios.title, summary, radios.head, &radios.rows);
        writeln!(out, "{facts}").ok();
        for card in &view.interfaces {
            writeln!(out, "\n{}", interface_facts(paint, card)).ok();
        }

        write!(out, "{list}").ok();

        if let Some(hint) = radios.hint {
            writeln!(out, "\n{}", paint.hint(&hint)).ok();
        }

        for (what, raw) in &view.reasons {
            failure(out, what, raw);
        }
    }

    if view.unread.is_empty() {
        Exit::Success
    } else {
        Exit::Failure
    }
}
