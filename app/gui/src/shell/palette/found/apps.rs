use egui::Context;
use toolbox_core::features::discover::package;
use toolbox_core::features::search::Needle;
use toolbox_core::features::search::ranked::top;
use toolbox_plan::PackageName;

use super::{CAP, Look, Part, Pick, Row, Section};
use crate::command::DiscoverCommand;
use crate::page::PageId;
use crate::pages::PageContext;
use crate::pages::discover::category_title;

pub fn found(
    ctx: &Context,
    context: &PageContext<'_>,
    needle: &Needle,
    query: &str,
) -> Option<Part> {
    let offered = context
        .catalogs
        .apps
        .apps
        .offered(context.snapshot, context.changes);

    let ranked = offered.filter_map(|(_, hit)| {
        let rank = hit.rank(needle)?;
        let title = hit.entry.title();
        let kept = package(hit.entry).map_or_else(|| hit.entry.id(), PackageName::as_str);

        let row = Row {
            name: title.to_owned(),
            detail: category_title(ctx, hit.category.id(), hit.category.title()),
            shortcut: None,
            section: Section::Apps,
            look: Look::Letter {
                package: kept.to_owned(),
            },
            pick: Pick::Run(DiscoverCommand::Reveal(title.to_owned()).into()),
        };

        Some((rank, row))
    });

    let all = DiscoverCommand::Reveal(query.trim().to_owned()).into();
    Some(Part::capped(top(ranked, CAP)?, PageId::Discover, all))
}
