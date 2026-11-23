use std::sync::Arc;

use egui::{Color32, Galley, Rect, Sense, Ui, Vec2, pos2, vec2};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{self, PAGE_TITLE, Weight};
use toolbox_plan::PackageName;

use crate::artwork::Artwork;
use crate::pages::PageContext;
use crate::pages::rows::{Initial, package_picture};

/// The width of the square [`title_picture`] draws and the gap after it.
pub(super) fn title_picture_width(ui: &Ui) -> f32 {
    capital_letter(ui).rect.height() + ui.tokens().sizes.picture_gap
}

/// The capital `H` in the page title's font, which sizes the picture.
fn capital_letter(ui: &Ui) -> Arc<Galley> {
    let font = fonts::font(ui.style(), Weight::Regular, PAGE_TITLE);
    ui.painter()
        .layout_no_wrap("H".to_owned(), font, Color32::PLACEHOLDER)
}

/// As tall as the capitals and starting on their top, so it stands level with
/// the name, which `page_header` centres on the toolbar row.
pub(super) fn title_picture(ui: &mut Ui, context: &PageContext<'_>, package: &PackageName) {
    let capital = capital_letter(ui);

    let size = capital.rect.height();
    let row = ui.tokens().sizes.toolbar_height;
    let top = (row - size) / 2.0 + capital.mesh_bounds.top();
    let (room, _) = ui.allocate_exact_size(vec2(size, top + size), Sense::hover());
    let slot = Rect::from_min_size(pos2(room.left(), room.top() + top), Vec2::splat(size));
    package_picture(
        ui,
        context.service::<Artwork>().pictures(),
        Some(package),
        Initial::named(package),
        slot,
    );

    ui.add_space(ui.tokens().sizes.picture_gap);
}
