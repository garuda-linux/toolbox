#[cfg(test)]
mod tests;

use egui::{Response, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    SearchTexts, icon_button_width, page_header, refresh_button_busy, search_field, segmented,
    segmented_width,
};

use crate::i18n;

/// The narrowest a toolbar's search field gets before a sort on its right
/// gives way, so the field keeps its glyph and clear button in view.
const MIN_SEARCH_WIDTH: f32 = 120.0;

/// A tab of a page, named by its text id.
pub trait TabText: Copy + PartialEq {
    fn text_id(self) -> &'static str;
}

#[derive(Clone, Copy, Debug)]
pub struct TabbedIds<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    /// What screen readers call the row of tabs.
    pub tabs: &'a str,
    pub refresh: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tabbed<T> {
    pub picked: Option<T>,
    pub refresh: bool,
}

pub fn title_header(ui: &mut Ui, title: &str, subtitle: &str) {
    let ctx = ui.ctx().clone();
    page_header(
        ui,
        &i18n::text(&ctx, title),
        &i18n::text(&ctx, subtitle),
        0.0,
        |_| {},
    );
}

/// Returns whether Read again was clicked or F5 pressed. While `reading`,
/// neither starts a second read.
pub fn refresh_header_reading(
    ui: &mut Ui,
    (title, subtitle): (&str, &str),
    refresh: &str,
    reading: bool,
) -> bool {
    let ctx = ui.ctx().clone();
    let mut pressed = false;
    page_header(
        ui,
        &i18n::text(&ctx, title),
        &i18n::text(&ctx, subtitle),
        icon_button_width(ui),
        |ui| pressed = read_again(ui, &i18n::text(&ctx, refresh), reading),
    );

    pressed
}

/// The tabs of a page and its Read again, which spins while `reading`.
pub fn tabbed_header_reading<T>(
    ui: &mut Ui,
    ids: TabbedIds<'_>,
    tabs: &[T],
    current: T,
    reading: bool,
) -> Tabbed<T>
where
    T: TabText,
{
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, ids.title);
    let subtitle = i18n::text(&ctx, ids.subtitle);
    tabbed_header_of(ui, (&title, &subtitle), ids, (tabs, current), reading)
}

/// A tabbed header whose title and subtitle are given as words rather
/// than text ids, such as a package's name and description. `ids.title`
/// and `ids.subtitle` are not read.
pub fn tabbed_header_of<T>(
    ui: &mut Ui,
    (title, subtitle): (&str, &str),
    ids: TabbedIds<'_>,
    (tabs, current): (&[T], T),
    reading: bool,
) -> Tabbed<T>
where
    T: TabText,
{
    let ctx = ui.ctx().clone();
    let names: Vec<_> = tabs
        .iter()
        .map(|tab| (*tab, i18n::text(&ctx, tab.text_id())))
        .collect();

    let options: Vec<(T, &str)> = names.iter().map(|(tab, name)| (*tab, &**name)).collect();
    let labels: Vec<&str> = options.iter().map(|(_, label)| *label).collect();
    let width = tabs_and_refresh_width(ui, &labels);
    let mut done = Tabbed {
        picked: None,
        refresh: false,
    };

    page_header(ui, title, subtitle, width, |ui| {
        done.picked = segmented(ui, ids.tabs, &i18n::text(&ctx, ids.tabs), &options, current).value;
        ui.add_space(ui.tokens().sizes.card_gap);
        done.refresh = read_again(ui, &i18n::text(&ctx, ids.refresh), reading);
    });

    done
}

/// A page's search field, `search_width` wide and never wider than the room
/// left of the row.
pub fn search(ui: &mut Ui, text: &mut String, name: &str, hint: &str) -> Response {
    search_beside(ui, text, name, hint, 0.0)
}

/// [`search`] on a toolbar that also draws a sort on its right, which
/// `beside` leaves room for.
pub fn search_beside_sort(ui: &mut Ui, text: &mut String, name: &str, hint: &str) -> Response {
    let sizes = ui.tokens().sizes;
    search_beside(ui, text, name, hint, sizes.sort_width + sizes.card_gap)
}

fn search_beside(ui: &mut Ui, text: &mut String, name: &str, hint: &str, beside: f32) -> Response {
    let ctx = ui.ctx().clone();
    let (name, hint, clear) = (
        i18n::text(&ctx, name),
        i18n::text(&ctx, hint),
        i18n::text(&ctx, "search-clear"),
    );

    let texts = SearchTexts {
        name: &name,
        hint: &hint,
        clear: &clear,
    };

    let room = ui.available_width() - beside;
    let width = field_width(room, ui.tokens().sizes.search_width);
    search_field(ui, texts.name, text, texts, width)
}

/// `wanted`, or less when the `room` left of the row holds no more.
pub const fn field_width(room: f32, wanted: f32) -> f32 {
    wanted.min(room).max(MIN_SEARCH_WIDTH)
}

fn tabs_and_refresh_width(ui: &Ui, labels: &[&str]) -> f32 {
    segmented_width(ui, labels) + ui.tokens().sizes.card_gap + icon_button_width(ui)
}

/// While a read runs the button spins and is named Reading, and a click or
/// F5 starts no second read, see `refresh_button_busy`.
fn read_again(ui: &mut Ui, name: &str, reading: bool) -> bool {
    let busy = reading.then(|| {
        let ctx = ui.ctx().clone();
        i18n::text(&ctx, "common-reading").into_owned()
    });

    refresh_button_busy(ui, name, busy.as_deref())
        .value
        .is_some()
}
