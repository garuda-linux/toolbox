//! System Tweaks: the system-wide choices, then the settings catalog,
//! under one search and one row of chips.

mod rows;
#[cfg(test)]
mod tests;

use egui::{Context, Ui};
use toolbox_core::features::tweaks::view::Choices;
use toolbox_system::Host;

use super::PageContext;
use super::catalog::{CatalogKind, CatalogPage};
use super::toggles::{LeadChip, LeadSections};
use crate::catalogs::CatalogId;
use crate::i18n;

const SHELL_NETWORK: &str = "shell-network";
const GARUDA_UPDATE: &str = "garuda-update";

pub type TweaksPage = CatalogPage<Tweaks>;

#[derive(Debug)]
pub struct Tweaks;

/// The choices as sections before the catalog's.
#[derive(Clone, Copy, Debug)]
pub struct SystemChoices;

impl CatalogKind for Tweaks {
    const CATALOG: CatalogId = CatalogId::Settings;
    const TITLE: &'static str = "page-system-settings";
    const SUBTITLE: &'static str = "page-system-settings-subtitle";
    type Lead = SystemChoices;
    const LEAD: SystemChoices = SystemChoices;

    fn span() -> tracing::Span {
        tracing::trace_span!("tweaks ui")
    }
}

impl LeadSections for SystemChoices {
    type Data = Choices;

    /// The login shell shown is the one of the account the app runs for.
    fn data(&self, context: &PageContext<'_>) -> Choices {
        let user = context
            .snapshot
            .part::<Host>()
            .and_then(|host| host.user.as_ref());

        Choices::of(context.snapshot, context.changes, user)
    }

    fn chips(&self, ctx: &Context, data: &Choices, query: &str) -> Vec<LeadChip> {
        let shell_network = LeadChip::new(
            ctx,
            SHELL_NETWORK,
            i18n::text(ctx, "tweaks-shell-network"),
            rows::shell_network_tally(ctx, data, query),
        );

        let garuda_update = data.flags.as_ref().map(|flags| {
            LeadChip::new(
                ctx,
                GARUDA_UPDATE,
                i18n::text(ctx, "tweaks-garuda-update"),
                rows::garuda_update_tally(ctx, flags, query),
            )
        });

        [Some(shell_network), garuda_update]
            .into_iter()
            .flatten()
            .collect()
    }

    fn unread(&self, data: &Choices) -> usize {
        rows::unread(data)
    }

    fn show(&self, ui: &mut Ui, on: (&PageContext<'_>, &Choices), id: &str, query: &str) -> bool {
        let (context, choices) = on;
        match id {
            SHELL_NETWORK => rows::shell_network(ui, context, choices, query),
            GARUDA_UPDATE => rows::garuda_update(ui, context, choices, query),
            _ => false,
        }
    }
}
