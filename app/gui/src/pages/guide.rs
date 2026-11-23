//! The User guide page: the pages of `docs/src`, built into the app.
//!
//! Three panes: the guide's pages at the left, the open one in the
//! middle, its headings at the right while the window is wide enough. A
//! search over every page takes the middle while it holds text.

#[cfg(test)]
mod tests;

mod article;
mod contents;
mod markdown;
mod outline;
mod picture;
mod results;

use egui::{Rect, ScrollArea, Ui, UiBuilder, pos2};
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_gui_kit::widgets::{BodyShape, page_toolbar};
use toolbox_guide::Hit;

use super::header::{search, title_header};
use super::{PageContext, PageView};
use crate::command::AppCommand;
use crate::i18n::Arg;
use crate::{i18n, links};

/// The widest the three panes get together.
const WIDTH: f32 = 1240.0;
const CONTENTS_WIDTH: f32 = 232.0;
const OUTLINE_WIDTH: f32 = 208.0;
const PANE_GAP: f32 = 32.0;
/// The narrowest the open page gets before "On this page" leaves.
const READER_MIN: f32 = 560.0;
/// Kept free at the right of a pane, so its scroll bar never covers text.
const SCROLL_ROOM: f32 = 12.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Jump {
    Page {
        file: String,
        anchor: Option<String>,
    },
    Anchor(String),
    Web(String),
}

#[derive(Debug)]
pub struct GuidePage {
    /// The open page's file under `docs/src`.
    file: String,
    /// The heading to bring into view, until it is drawn.
    target: Option<String>,
    /// The open page changed, so it starts at its top.
    fresh: bool,
    search: String,
    /// The last search and its hits, kept while the search stays.
    found: (String, Vec<Hit>),
    /// A page asked for that the guide lacks, until the toast says so.
    missing: Option<String>,
    /// The source and description of the picture shown large.
    lightbox: Option<(String, String)>,
}

impl Default for GuidePage {
    fn default() -> Self {
        Self {
            file: toolbox_guide::FIRST_PAGE.to_owned(),
            target: None,
            fresh: true,
            search: String::new(),
            found: (String::new(), Vec::new()),
            missing: None,
            lightbox: None,
        }
    }
}

impl GuidePage {
    /// Opens the guide page `file`, at `anchor` when given. A file the
    /// guide lacks opens the first page, and a toast says so.
    pub fn show(&mut self, file: &str, anchor: Option<&str>) {
        let known = toolbox_guide::embedded().page(file).is_some();
        file.clone_into(&mut self.file);
        if !known {
            tracing::warn!("the guide has no page {file}");
            self.missing = Some(file.to_owned());
            toolbox_guide::FIRST_PAGE.clone_into(&mut self.file);
        }

        self.target = anchor.filter(|_| known).map(str::to_owned);
        self.fresh = true;
        self.search.clear();
    }

    fn results(&mut self, ui: &mut Ui, middle: Rect) -> Option<Jump> {
        let query = self.search.trim();
        if self.found.0 != query {
            self.found = (query.to_owned(), toolbox_guide::embedded().search(query));
        }

        let hits = &self.found.1;
        pane(ui, middle, ("guide-results", query), |places| {
            results::show(places, hits, query)
        })
    }

    fn follow(&mut self, ctx: &egui::Context, jump: Jump) {
        match jump {
            Jump::Page { file, anchor } => self.show(&file, anchor.as_deref()),
            Jump::Anchor(anchor) => self.target = Some(anchor),
            Jump::Web(address) => links::open(ctx, &address),
        }
    }
}

impl PageView for GuidePage {
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        title_header(ui, "page-guide", "page-guide-subtitle");
        page_toolbar(
            ui,
            self,
            |page, ui| _ = search(ui, &mut page.search, "guide-search", "guide-search-hint"),
            |_, _| {},
        );
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("guide ui").entered();
        let guide = toolbox_guide::embedded();
        if let Some(file) = self.missing.take() {
            let page = [("page", Arg::from(&file))];
            context.commands.send(AppCommand::Toast {
                kind: ToastKind::Error,
                text: i18n::text_with(ui.ctx(), "guide-page-missing", &page).into_owned(),
            });
        }

        // Shown before the article, so the click that asked for it is not
        // also taken as a click on the large picture or beside it.
        let open = self
            .lightbox
            .as_ref()
            .is_some_and(|(src, alt)| picture::lightbox(ui.ctx(), src, alt));

        if !open {
            self.lightbox = None;
        }

        let full = below_purpose(ui);

        let reader = PANE_GAP.mul_add(-2.0, full.width() - CONTENTS_WIDTH - OUTLINE_WIDTH);
        let outlined = reader >= READER_MIN;

        let left = Rect::from_min_max(full.min, pos2(full.left() + CONTENTS_WIDTH, full.bottom()));
        let right_edge = if outlined {
            full.right() - OUTLINE_WIDTH - PANE_GAP
        } else {
            full.right()
        };

        let middle = Rect::from_min_max(
            pos2(left.right() + PANE_GAP, full.top()),
            pos2(right_edge, full.bottom()),
        );

        let mut jump = pane(ui, left, "guide-contents", |list| {
            contents::show(list, guide, &self.file, self.fresh)
        });

        let query = self.search.trim();
        if query.is_empty() {
            let shown = article::show(ui, middle, guide, self, context);
            jump = jump.or(shown.jump);

            // Every heading of the page is drawn each frame, so a target
            // not reached in one is not on the page.
            self.target = None;

            if shown.enlarged.is_some() {
                self.lightbox = shown.enlarged;
                ui.ctx().request_repaint();
            }

            if outlined && let Some(page) = guide.page(&self.file) {
                let right =
                    Rect::from_min_max(pos2(full.right() - OUTLINE_WIDTH, full.top()), full.max);

                let current = shown.current.as_deref();
                let picked = pane(ui, right, "guide-outline", |headings| {
                    outline::show(headings, page, current)
                });

                jump = jump.or(picked);
            }
        } else {
            let picked = self.results(ui, middle);
            jump = jump.or(picked);
        }

        if let Some(jump) = jump {
            self.follow(ui.ctx(), jump);
        }

        ui.advance_cursor_after_rect(full);
    }

    fn shape(&self) -> BodyShape {
        BodyShape::Panes { width: WIDTH }
    }
}

fn pane<R>(
    ui: &mut Ui,
    rect: Rect,
    salt: impl std::hash::Hash + std::fmt::Debug,
    body: impl FnOnce(&mut Ui) -> R,
) -> R {
    let mut inner = ui.new_child(UiBuilder::new().max_rect(rect));
    ScrollArea::vertical()
        .id_salt(salt)
        .auto_shrink(false)
        .show(&mut inner, |scrolled| {
            scrolled.set_width(scrolled.available_width() - SCROLL_ROOM);
            body(scrolled)
        })
        .inner
}

/// The body below the purpose, which the body draws first, so the panes
/// do not overlap its line.
fn below_purpose(ui: &Ui) -> Rect {
    Rect::from_min_max(
        pos2(ui.max_rect().left(), ui.cursor().top()),
        ui.max_rect().max,
    )
}
