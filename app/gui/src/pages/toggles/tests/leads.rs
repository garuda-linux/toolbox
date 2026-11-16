//! The lead sections' chips, built when the system, the queue, the search
//! or the look change.

use egui::{Context, Ui};
use toolbox_core::features::search::Needle;
use toolbox_core::features::toggles::shown::Tally;

use super::{TogglePage, shipped};
use crate::catalogs::CatalogId;
use crate::pages::toggles::{LeadChip, LeadSections};
use crate::pages::{PageContext, PageView};

#[derive(Clone, Copy, Debug)]
struct OneLead;

impl LeadSections for OneLead {
    type Data = ();

    fn data(&self, _context: &PageContext<'_>) {}

    fn chips(&self, ctx: &Context, (): &(), _query: &Needle) -> Vec<LeadChip> {
        let tally = Tally { on: 0, total: 1 };
        vec![LeadChip::new(ctx, "lead", "Lead".into(), (tally, true))]
    }

    fn show(&self, _ui: &mut Ui, _: (&PageContext<'_>, &()), _id: &str, _query: &Needle) -> bool {
        false
    }
}

struct Led(TogglePage);

impl PageView for Led {
    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        self.0.ui(ui, context, (&OneLead, &()));
    }
}

#[test]
fn a_frame_with_the_same_system_builds_no_lead_chip() {
    let page = Led(TogglePage::new(
        CatalogId::Settings,
        "page-system-settings",
        "",
    ));

    let mut harness = shipped().harness(page);
    harness.run();
    let built = harness.state().page.0.leads.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.0.leads.builds(), built);

    harness.state_mut().fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.0.leads.builds(), built + 1);
}
