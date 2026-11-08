//! System Tweaks: the system-wide choices, then the settings catalog,
//! under one search and one row of chips.

mod rows;
#[cfg(test)]
mod tests;
pub mod view;

use egui::{Context, Ui};
use toolbox_core::features::toggles::shown::Tally;
use toolbox_system::Host;
use view::Choices;

use super::PageContext;
use super::catalog::{CatalogKind, CatalogPage};
use super::toggles::{LeadChip, LeadSections};
use crate::catalogs::CatalogId;
use crate::i18n;

/// The ids of the choices' sections, for their chips.
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
    fn chips(&self, ctx: &Context, context: &PageContext<'_>, query: &str) -> Vec<LeadChip> {
        let choices = choices(context);
        let shell_network = LeadChip::new(
            ctx,
            SHELL_NETWORK,
            i18n::text(ctx, "tweaks-shell-network"),
            rows::shell_network_tally(ctx, &choices, query),
        );

        let garuda_update = choices.flags.map(|flags| {
            let tally = flags
                .iter()
                .filter(|(flag, _)| {
                    rows::matches(ctx, &format!("tweaks-flag-{}", flag.as_str()), query)
                })
                .map(|(_, flip)| Tally {
                    on: usize::from(flip.on),
                    total: 1,
                })
                .sum();

            LeadChip::new(
                ctx,
                GARUDA_UPDATE,
                i18n::text(ctx, "tweaks-garuda-update"),
                tally,
            )
        });

        [Some(shell_network), garuda_update]
            .into_iter()
            .flatten()
            .collect()
    }

    fn show(&self, ui: &mut Ui, context: &PageContext<'_>, id: &str, query: &str) -> bool {
        let choices = choices(context);
        match id {
            SHELL_NETWORK => rows::shell_network(ui, context, &choices, query),
            GARUDA_UPDATE => rows::garuda_update(ui, context, &choices, query),
            _ => false,
        }
    }
}

/// The login shell shown is the one of the account the app runs for.
fn choices(context: &PageContext<'_>) -> Choices {
    let user = context
        .snapshot
        .part::<Host>()
        .and_then(|host| host.user.as_ref());

    Choices::of(context.snapshot, context.changes, user)
}
