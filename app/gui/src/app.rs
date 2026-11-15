//! The root of the UI: the persisted [`State`], the command bus and the
//! frame loop.

mod answers;
mod bootrepair;
mod closing;
mod commands;
mod configs;
mod console;
mod fold;
mod home;
mod instance;
mod maintenance;
mod network;
mod preferences;
mod services;
mod setup;
#[cfg(test)]
mod tests;
mod units;
mod users;

use std::sync::Arc;

use eframe::CreationContext;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use toolbox_core::{ChangeView, Snapshot};
use toolbox_gui_kit::desktop::FontSpec;
use toolbox_gui_kit::icons::ThemeIcons;
use toolbox_gui_kit::row_cache::RowCache;
use toolbox_gui_kit::toasts::Toasts;
use toolbox_gui_kit::{fonts, icons};

use self::preferences::Preferences;
pub use self::preferences::PreferencesSetup;
pub use self::setup::{FIXED_VERSION, Setup};
use crate::apply::ApplyHandle;
use crate::artwork::Artwork;
use crate::boot::Boot;
use crate::bootrepair::BootRepair;
use crate::catalogs::Catalogs;
use crate::command::{CommandBus, CommandReceiver, CommandSender};
use crate::configs::Configs;
use crate::console::Console;
use crate::desktop::DesktopLook;
use crate::diagnostics::Diagnostics;
use crate::home::{Home, HomeReaders};
use crate::instance::Arrivals;
use crate::kernels::Kernels;
use crate::launch::Launcher;
use crate::maintenance::Maintenance;
use crate::network::Network;
use crate::notifications::Notifier;
use crate::packages::Packages;
use crate::page::PageId;
use crate::page_service::Services;
use crate::pages::home::HomeProblem;
use crate::pages::{PageContext, Pages, Plans};
use crate::pictures::FALLBACK_THEME;
use crate::queue::Queue;
use crate::reader::fresh;
use crate::settings::{MotionChoice, ThemeChoice};
use crate::share::ShareLink;
use crate::shell::{ChangesState, Palette};
use crate::snapshot::{Reading, SnapshotHandle};
use crate::toggle_pages::TogglePages;
use crate::units::Units;
use crate::users::Users;
use crate::{profiling, shell, waking};

/// What survives a restart. Everything else is rebuilt from the system.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct State {
    page: PageId,
    sidebar_expanded: bool,
    console_open: bool,
    theme: ThemeChoice,
    motion: MotionChoice,
    /// The pages used last, newest first, for Home's shortcuts.
    recent: Vec<PageId>,
    /// The Setup Assistant reached its end once.
    setup_finished: bool,
}

pub struct App {
    state: State,
    look: Option<watch::Receiver<DesktopLook>>,
    /// The fonts in use, so a look that names the same ones does not
    /// rebuild egui's font atlas. The last: the emoji font is in.
    fonts: (Option<FontSpec>, Option<FontSpec>, bool),
    /// The icon theme's glyphs in use. Rasterizing them takes a while, and
    /// most look changes keep the theme.
    glyphs: Option<Arc<ThemeIcons>>,
    sender: CommandSender,
    receiver: CommandReceiver,
    palette: Palette,
    queue: Queue,
    /// The queue's rows for the shell, built once per queue revision.
    views: RowCache<u64, Vec<ChangeView>>,
    changes_ui: ChangesState,
    /// What a shortcut changed this frame, for the shell to jump to.
    snap: shell::Snap,
    fold: fold::Fold,
    console: Console,
    apply: Option<ApplyHandle>,
    notifier: Option<Arc<dyn Notifier>>,
    snapshot: Option<SnapshotHandle>,
    reading: Reading,
    /// The window had the focus last frame, so regaining it reads the
    /// system again.
    focused: bool,
    catalogs: Arc<Catalogs>,
    desktops: Vec<String>,
    version: &'static str,
    dry_run: bool,
    launcher: Option<Arc<dyn Launcher>>,
    services: Services,
    output_share: Option<ShareLink>,
    /// Told on exit, so no raise touches the closing window.
    arrivals: Option<Arc<Arrivals>>,
    plans: Plans,
    closing: closing::Closing,
    /// The probed snapshot joined with what the pages read beside it: the
    /// lock flags, the AUR's names and the settings of systemd-boot.
    seen: Arc<Snapshot>,
    pages: Pages,
    /// Notes about background results, drawn over the page.
    toasts: Toasts,
    prefs: Preferences,
}

impl Default for State {
    fn default() -> Self {
        Self {
            page: PageId::default(),
            sidebar_expanded: true,
            console_open: false,
            theme: ThemeChoice::default(),
            motion: MotionChoice::default(),
            recent: Vec::new(),
            setup_finished: false,
        }
    }
}

impl App {
    #[must_use = "the app does nothing until eframe runs it"]
    #[expect(
        clippy::too_many_lines,
        reason = "one line per field of the app, which groups would only hide"
    )]
    pub fn new(cc: &CreationContext<'_>, setup: Setup) -> Self {
        install(cc, setup.arrivals.as_deref());

        let prefs = Preferences::new(setup.preferences, setup.system_language);
        let catalogs = Arc::new(setup.catalogs);
        let CommandBus { sender, receiver } = setup.commands.unwrap_or_default();
        let mut maintenance = Maintenance::new(setup.maintenance, cc.egui_ctx.clone());
        maintenance.set_catalogs(Arc::clone(&catalogs));
        maintenance.set_launcher(setup.launcher.clone());
        let readers = HomeReaders {
            status: setup.status,
            facts: setup.facts,
            news: setup.news,
            programs: setup.programs,
        };

        let mut home = Home::new(readers, setup.power, waking::repaint(cc.egui_ctx.clone()));
        home.set_problem(
            (setup.queue_lost > 0).then_some(HomeProblem::QueueLost(setup.queue_lost)),
        );
        let app = Self {
            pages: preferences::pages_for(&prefs),
            prefs,
            state: restored(cc, setup.open),
            look: setup.look,
            fonts: (None, None, false),
            glyphs: None,
            sender,
            receiver,
            palette: Palette::default(),
            queue: setup.queue,
            views: RowCache::new(),
            changes_ui: ChangesState::default(),
            snap: shell::Snap::default(),
            fold: fold::Fold::default(),
            console: Console::knowing(ShareLink::known_of(setup.output_share.as_ref())),
            apply: setup.apply,
            notifier: setup.notifier,
            snapshot: setup.snapshot,
            reading: Reading::default(),
            focused: true,
            catalogs: Arc::clone(&catalogs),
            desktops: setup.desktops,
            version: setup.version,
            dry_run: setup.dry_run,
            launcher: setup.launcher,
            services: Services {
                home,
                maintenance,
                diagnostics: Diagnostics::new(setup.diagnostics),
                boot: Boot::new(setup.boot, cc.egui_ctx.clone()),
                network: Network::new(setup.network, cc.egui_ctx.clone()),
                packages: Packages::new(setup.packages, cc.egui_ctx.clone()),
                users: Users::new(setup.users, cc.egui_ctx.clone()),
                configs: Configs::new(setup.configs, cc.egui_ctx.clone()),
                boot_repair: BootRepair::new(setup.boot_repair, cc.egui_ctx.clone()),
                toggle_pages: TogglePages::new(catalogs),
                kernels: Kernels::new(setup.dkms),
                units: Units::new(setup.units),
                artwork: Artwork::new(setup.pictures),
                ..Services::default()
            },
            output_share: setup.output_share,
            arrivals: setup.arrivals,
            plans: Plans::default(),
            closing: closing::Closing::default(),
            seen: Arc::default(),
            toasts: Toasts::default(),
        };

        app.apply_language(&cc.egui_ctx);
        app.apply_palettes(&cc.egui_ctx);
        app.apply_choices(&cc.egui_ctx);
        crate::page_ui::offer_boot_repair(&cc.egui_ctx, app.services.boot_repair.offered());
        app
    }

    fn take_look(&mut self, ctx: &egui::Context, desktop: &DesktopLook) {
        let look = &desktop.look;
        self.prefs.desktop = (look.theme, look.reduced_motion);
        self.prefs.palettes = (look.dark, look.light);
        self.apply_palettes(ctx);
        self.apply_choices(ctx);
        let same_glyphs = match (&self.glyphs, &desktop.icons) {
            (Some(old), Some(new)) => Arc::ptr_eq(old, new),
            (old, new) => old.is_none() && new.is_none(),
        };

        if !same_glyphs {
            self.glyphs.clone_from(&desktop.icons);
            icons::use_theme(ctx, desktop.icons.clone());
            icons::warm(ctx);
        }

        let theme = desktop
            .icons
            .as_ref()
            .map_or(FALLBACK_THEME, |icons| icons.theme());

        self.services
            .artwork
            .use_theme(desktop.app_icons.as_ref(), theme);

        let emoji = desktop.emoji_font.is_some();
        let fonts = (look.font.clone(), look.fixed_font.clone(), emoji);
        if fonts != self.fonts {
            ctx.set_fonts(crate::desktop::font_definitions(desktop));
            fonts::apply_sizes(
                ctx,
                fonts.0.as_ref().map(|font| font.points),
                fonts.1.as_ref().map(|font| font.points),
            );
            self.fonts = fonts;
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        profiling::frame_mark();
        let _logic = tracing::trace_span!("app logic").entered();
        if let Some(desktop) = self.look.as_mut().and_then(fresh) {
            self.take_look(ctx, &desktop);
        }

        let width = ctx.content_rect().width();
        self.state.sidebar_expanded = self.fold.fit(width, self.state.sidebar_expanded);
        self.take_reading(ctx);
        self.take_services(ctx);
        self.take_reports(ctx);
        self.answer_queue(ctx);
        self.guard_close(ctx);
        self.run_all(ctx, Self::pressed(ctx));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let _ui = tracing::trace_span!("app ui").entered();
        self.remember_page();
        let changes = self
            .views
            .get(&[], self.queue.revision(), || self.queue.views());

        let held = self.apply_held();
        let snapshot = self.current_snapshot();
        let context = PageContext {
            snapshot: &snapshot,
            probed: self.reading.get().is_some(),
            changes: self.queue.changes(),
            revision: self.queue.revision(),
            catalogs: &self.catalogs,
            busy: self.console.running(),
            console_open: self.state.console_open,
            home: self
                .services
                .home
                .shown(&self.state.recent, self.state.setup_finished),
            services: &self.services,
            plans: self.plans,
            desktops: &self.desktops,
            version: self.version,
            commands: &self.sender,
            preferences: preferences::view(&self.prefs, (self.state.theme, self.state.motion)),
        };

        let view = shell::View {
            pages: &mut self.pages,
            context: &context,
            dry_run: self.dry_run,
            page: self.state.page,
            sidebar_expanded: &mut self.state.sidebar_expanded,
            console_open: self.state.console_open,
            snap: std::mem::take(&mut self.snap),
            console: &mut self.console,
            palette: &mut self.palette,
            changes: &changes,
            changes_ui: &mut self.changes_ui,
            reading: &self.reading,
            held,
            toasts: &mut self.toasts,
        };

        shell::show(ui, view, &self.sender);
        self.ask_before_close(ui.ctx());
    }

    /// eframe's default is a translucent grey, which shows at the edges
    /// while the window grows.
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn on_exit(&mut self) {
        self.leave();
    }
}

/// The state of the last run, on the page a start option names.
fn restored(cc: &CreationContext<'_>, open: Option<PageId>) -> State {
    let mut state: State = cc
        .storage
        .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
        .unwrap_or_default();

    if let Some(page) = open {
        state.page = page;
    }

    state
}

/// Styles, live blur, and the window for later starts.
fn install(cc: &CreationContext<'_>, arrivals: Option<&Arrivals>) {
    toolbox_gui_kit::install(&cc.egui_ctx);
    if let Some(state) = cc.wgpu_render_state.clone() {
        toolbox_gui_kit::frost::install(&cc.egui_ctx, state);
    }

    if let Some(arrivals) = arrivals {
        arrivals.set_context(cc.egui_ctx.clone());
    }
}
