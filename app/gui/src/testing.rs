//! One page alone in the kittest harness, drawn from fixtures.
//!
//! The page reads a [`PageFixture`] instead of the app's services, and
//! what it sends waits in [`PageFixture::sent`] for the test.

#[cfg(test)]
pub mod captures;
mod ink;
mod rows;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::accesskit::Role;
use egui::{CentralPanel, Frame, Rect, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
pub use ink::{drawing_top, letters, letters_top};
pub use rows::{
    assert_rows_hold_their_content, assert_rows_hold_their_parts, assert_text_beside_picture,
    picture_beside, text_left,
};
use toolbox_config::Settings;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui_kit::Loadable;
use toolbox_gui_kit::widgets::{ButtonKind, PageParts, page_layout_shaped};
use toolbox_news::News;
pub use toolbox_services::testing::{PATIENCE, poll, until};
use toolbox_system::{Facts, Status};

use crate::catalogs::Catalogs;
use crate::command::{self, Command, CommandReceiver, CommandSender};
use crate::page::PageId;
use crate::page_service::Services;
use crate::pages::home::{HomeFacts, HomeProblem};
use crate::pages::kernels::view::DkmsReading;
use crate::pages::{PageContext, PageView, Plans, Refusals};
use crate::programs::Programs;
use crate::settings::{MotionChoice, SettingsView, ThemeChoice};
use crate::units::Units;

/// The window a page is drawn in, as the app's own tests use.
const WIDTH: f32 = 1024.0;
const HEIGHT: f32 = 900.0;

pub const PLAN_RUNNING: &str = "Changes are being applied. This waits until they are done.";
/// The collapsed sidebar's width, its margins included.
pub const COLLAPSED_SIDEBAR: f32 = crate::shell::COLLAPSED_WIDTH;
/// How far a search field's text starts from the field's left edge.
const SEARCH_INSET: f32 = 48.0;

/// Everything a page reads, set by the test. Read and nothing pending
/// unless the test says otherwise.
#[expect(
    clippy::struct_excessive_bools,
    reason = "the fixture mirrors the flags of the page context one by one"
)]
#[derive(Debug)]
pub struct PageFixture {
    pub snapshot: Arc<Snapshot>,
    pub probed: bool,
    pub changes: ChangeSet,
    /// Raise it with every change to `changes` after the first frame, as
    /// the queue does.
    pub revision: u64,
    pub catalogs: Catalogs,
    pub refusals: Refusals,
    pub dkms: DkmsReading,
    pub busy: bool,
    pub console_open: bool,
    pub units: Units,
    /// `None` leaves Home without a status, as without a reader.
    pub status: Option<Loadable<Status>>,
    /// `None` leaves Home without its facts, as without a reader.
    pub facts: Option<Loadable<Facts>>,
    pub news: Option<Loadable<News>>,
    pub recent: Vec<PageId>,
    pub setup_finished: bool,
    pub programs: Option<Programs>,
    pub problem: Option<HomeProblem>,
    pub services: Services,
    pub plans: Plans,
    pub desktops: Vec<String>,
    pub settings: Settings,
    /// The page sends here, [`PageFixture::sent`] reads it back.
    pub commands: CommandSender,
    pub receiver: CommandReceiver,
}

#[derive(Debug)]
pub struct PageState<P> {
    pub page: P,
    pub fixture: PageFixture,
}

impl Default for PageFixture {
    fn default() -> Self {
        let (commands, receiver) = command::channel();
        Self {
            snapshot: Arc::default(),
            probed: true,
            changes: ChangeSet::default(),
            revision: 0,
            catalogs: Catalogs::default(),
            refusals: Refusals::default(),
            dkms: DkmsReading::default(),
            busy: false,
            console_open: false,
            units: Units::default(),
            status: None,
            facts: None,
            news: None,
            recent: Vec::new(),
            setup_finished: false,
            programs: None,
            problem: None,
            services: Services::default(),
            plans: Plans::default(),
            desktops: Vec::new(),
            settings: Settings::default(),
            commands,
            receiver,
        }
    }
}

impl PageFixture {
    pub fn with_snapshot(snapshot: Snapshot) -> Self {
        Self {
            snapshot: Arc::new(snapshot),
            ..Self::default()
        }
    }

    pub fn context(&self) -> PageContext<'_> {
        PageContext {
            snapshot: &self.snapshot,
            probed: self.probed,
            changes: &self.changes,
            revision: self.revision,
            catalogs: &self.catalogs,
            refusals: &self.refusals,
            dkms: &self.dkms,
            busy: self.busy,
            console_open: self.console_open,
            units: &self.units,
            home: HomeFacts {
                status: self.status.as_ref(),
                facts: self.facts.as_ref(),
                news: self.news.as_ref(),
                recent: &self.recent,
                setup_finished: self.setup_finished,
                programs: self.programs.as_ref(),
                programs_failed: false,
                problem: self.problem.as_ref(),
            },
            services: &self.services,
            plans: self.plans,
            desktops: &self.desktops,
            version: crate::FIXED_VERSION,
            app_icons: None,
            pictures: None,
            commands: &self.commands,
            preferences: SettingsView {
                settings: &self.settings,
                autostart: false,
                language: None,
                theme: ThemeChoice::default(),
                motion: MotionChoice::default(),
            },
        }
    }

    pub fn sent(&self) -> Vec<Command> {
        self.receiver.drain().map(|sent| sent.command).collect()
    }

    /// A harness that draws `page`, its header fixed above its scrolling
    /// body, as the app's window does, and nothing else. Run it before the
    /// first query.
    #[must_use = "a harness does nothing until it runs"]
    pub fn harness<P>(self, page: P) -> Harness<'static, PageState<P>>
    where
        P: PageView + 'static,
    {
        let state = PageState {
            page,
            fixture: self,
        };

        toolbox_gui_kit::testing::harness(WIDTH, HEIGHT).build_ui_state(
            move |ui, state: &mut PageState<P>| {
                let context = state.fixture.context();
                let window = toolbox_gui_kit::tokens::current(ui.ctx()).window;

                // The harness leaves a clear margin round the page. The app
                // fills it, and so must a blurred shot of a page alone.
                let screen = ui.ctx().content_rect();
                let back = egui::LayerId::background();
                ui.ctx()
                    .layer_painter(back)
                    .rect_filled(screen, 0.0, window);
                CentralPanel::default()
                    .frame(Frame::new().fill(window))
                    .show(ui, |ui| {
                        let parts = PageParts {
                            header: |shown: &mut P, area: &mut Ui| shown.header(area, &context),
                            body: |shown: &mut P, area: &mut Ui| shown.ui(area, &context),
                            footer: |shown: &mut P, area: &mut Ui| shown.footer(area, &context),
                        };

                        let shape = state.page.shape();
                        page_layout_shaped(ui, "page", &mut state.page, parts, shape);
                    });
            },
            state,
        )
    }
}

/// Draws the page of `fixture` with and without a running plan: while it
/// runs every switch on screen rests and the banner says why, without it
/// a switch takes clicks and no banner shows.
///
/// # Panics
///
/// When the page shows no switch, or the switches or the banner differ
/// from that.
pub fn assert_rows_wait_while_busy<P>(fixture: impl Fn() -> PageFixture, page: impl Fn() -> P)
where
    P: PageView + 'static,
{
    for busy in [true, false] {
        let mut harness = PageFixture { busy, ..fixture() }.harness(page());

        harness.run();

        let disabled: Vec<bool> = harness
            .query_all_by_role(Role::CheckBox)
            .map(|switch| switch.accesskit_node().is_disabled())
            .collect();

        assert!(!disabled.is_empty(), "no switch on screen");
        if busy {
            assert!(
                disabled.iter().all(|resting| *resting),
                "busy: {disabled:?}"
            );
        } else {
            assert!(
                disabled.iter().any(|resting| !resting),
                "idle: {disabled:?}"
            );
        }

        assert_eq!(
            harness.query_by_label(PLAN_RUNNING).is_some(),
            busy,
            "the banner"
        );
    }
}

/// Makes the body text `size` points, as a desktop with a larger font
/// does. Every other size of the type scale follows it.
pub fn body_font(ctx: &egui::Context, size: f32) {
    ctx.all_styles_mut(|style| {
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(size));
    });
}

/// What `measure` gives in a frame of the fallback look with the body
/// text `size` points, such as a page's row height.
pub fn measured(size: f32, measure: fn(&Ui) -> f32) -> f32 {
    let mut harness = toolbox_gui_kit::testing::harness(400.0, 200.0)
        .build_ui_state(move |ui, height: &mut f32| *height = measure(ui), 0.0);

    body_font(&harness.ctx, size);
    harness.run();
    *harness.state()
}

/// Where the node of `role` that `label` names is drawn.
///
/// # Panics
///
/// When no such node shows.
pub fn rect_of<S>(harness: &Harness<'_, S>, role: Role, label: &str) -> Rect {
    harness.get_by_role_and_label(role, label).rect()
}

/// Checks the page header of `title` with purpose `subtitle`.
///
/// `control` (when given) sits beside the title, not stacked under the
/// purpose. `search` (when given) starts the toolbar at the title's left
/// edge, below the purpose. The search's text starts after the field's
/// padding and magnifier, at most [`SEARCH_INSET`] in.
///
/// # Panics
///
/// When a node is missing or out of place.
pub fn assert_header<S>(
    harness: &Harness<'_, S>,
    (title, subtitle): (&str, &str),
    control: Option<Rect>,
    search: Option<Rect>,
) {
    let title = harness.get_by_role_and_label(Role::Heading, title).rect();
    let subtitle = harness.get_by_role_and_label(Role::Label, subtitle).rect();
    if let Some(control) = control {
        assert!(
            control.top() < subtitle.bottom(),
            "the header control at {control:?} is stacked under the purpose {subtitle:?}"
        );
    }

    if let Some(search) = search {
        let inset = search.left() - title.left();
        assert!(
            (0.0..=SEARCH_INSET).contains(&inset),
            "the search at {search:?} is not at the title's left edge {title:?}"
        );
        assert!(
            search.top() >= subtitle.bottom(),
            "the search is not under the purpose"
        );
    }
}

/// How strongly the button `label` calls for a click, read from the fill
/// under its middle.
///
/// # Panics
///
/// When no single button is named `label` or nothing is drawn under it.
pub fn button_kind<S>(harness: &Harness<'_, S>, label: &str) -> ButtonKind {
    kind_at(harness, rect_of(harness, Role::Button, label).center())
}

/// How strongly the button drawn at `at` calls for a click, read from the
/// fill there.
pub fn kind_at<S>(harness: &Harness<'_, S>, at: egui::Pos2) -> ButtonKind {
    let fill = toolbox_gui_kit::testing::fill_under(harness, at);
    let tokens = toolbox_gui_kit::tokens::current(&harness.ctx);
    if fill == Some(tokens.accent) {
        ButtonKind::Primary
    } else if fill == Some(tokens.error) {
        ButtonKind::Danger
    } else if fill == Some(tokens.field) {
        ButtonKind::Soft
    } else {
        ButtonKind::Ghost
    }
}
