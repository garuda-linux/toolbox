#[cfg(test)]
mod tests;

use egui::{Response, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    SearchTexts, icon_button_width, page_header, refresh_button, search_field, segmented,
    segmented_width,
};

use crate::i18n;

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

/// Returns whether Read again was clicked or F5 pressed.
pub fn refresh_header(ui: &mut Ui, title: &str, subtitle: &str, refresh: &str) -> bool {
    let ctx = ui.ctx().clone();
    let mut pressed = false;
    page_header(
        ui,
        &i18n::text(&ctx, title),
        &i18n::text(&ctx, subtitle),
        icon_button_width(ui),
        |ui| pressed = refresh_button(ui, &i18n::text(&ctx, refresh)),
    );

    pressed
}

pub fn tabbed_header<T>(ui: &mut Ui, ids: TabbedIds<'_>, tabs: &[T], current: T) -> Tabbed<T>
where
    T: TabText,
{
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, ids.title);
    let subtitle = i18n::text(&ctx, ids.subtitle);
    tabbed_header_of(ui, (&title, &subtitle), ids, tabs, current)
}

/// such as a package's name and description. `ids.title` and
/// `ids.subtitle` are not read.
pub fn tabbed_header_of<T>(
    ui: &mut Ui,
    (title, subtitle): (&str, &str),
    ids: TabbedIds<'_>,
    tabs: &[T],
    current: T,
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
        done.picked = segmented(ui, &i18n::text(&ctx, ids.tabs), &options, current);
        ui.add_space(ui.tokens().sizes.card_gap);
        done.refresh = refresh_button(ui, &i18n::text(&ctx, ids.refresh));
    });

    done
}

pub fn search(ui: &mut Ui, text: &mut String, name: &str, hint: &str) -> Response {
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

    let width = ui.tokens().sizes.search_width;
    search_field(ui, text, texts, width)
}

fn tabs_and_refresh_width(ui: &Ui, labels: &[&str]) -> f32 {
    segmented_width(ui, labels) + ui.tokens().sizes.card_gap + icon_button_width(ui)
}
