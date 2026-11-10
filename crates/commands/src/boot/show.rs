//! `boot show` and `boot entries`: the GRUB settings and the menu.

use std::io::Write as _;

use toolbox_cli::Exit;
use toolbox_cli::args::Read;
use toolbox_core::features::boot::shown;
use toolbox_core::{ChangeSet, Snapshot};

use super::listing::entry_listing;
use super::{BootView, read_menu};
use crate::out::Out;
use crate::settings::on_off_text;
use crate::{json, print, unavailable};

pub(super) fn show(read: Read, snapshot: &Snapshot, out: &mut Out<'_>) -> Exit {
    let Some(now) = shown(snapshot, &ChangeSet::default()) else {
        return unavailable("GRUB is not the boot loader here", out);
    };

    let view = BootView {
        default: now.default,
        remember: now.remember,
        timeout: now.timeout,
        parameters: now
            .parameters
            .map(|parameters| parameters.iter().map(ToString::to_string).collect()),
        submenus: now.submenus,
        os_prober: now.os_prober,
        background: now.background,
        theme: now.theme,
    };

    let text = if read.json {
        json(&view)
    } else {
        let timeout = view
            .timeout
            .map_or_else(String::new, |seconds| seconds.to_string());

        let parameters = view
            .parameters
            .as_ref()
            .map_or_else(String::new, |list| list.join(" "));

        let paint = out.paint();
        let facts = paint.facts(
            "Boot menu",
            &[
                ("default", &view.default),
                ("remember", on_off_text(view.remember)),
                ("timeout", &timeout),
                ("parameters", &parameters),
                ("submenus", on_off_text(view.submenus)),
                ("os-prober", on_off_text(view.os_prober)),
                ("background", &view.background),
                ("theme", &view.theme),
            ],
        );

        format!(
            "{facts}\n{}\n",
            paint.hint("garuda-toolbox boot set <setting> <value>")
        )
    };

    write!(out, "{text}").ok();
    Exit::Success
}

pub(super) fn entries(read: Read, out: &mut Out<'_>) -> Exit {
    match read_menu(out.settings().color) {
        Some(menu) => print(
            read,
            &toolbox_core::features::boot::view::entries(Some(&menu)),
            out,
            entry_listing,
        ),
        None => unavailable("the boot menu could not be read", out),
    }
}
