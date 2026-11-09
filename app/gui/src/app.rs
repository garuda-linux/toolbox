//! The root of the UI: the persisted [`State`], the command bus and the
//! frame loop.

mod boot;
mod bootrepair;
mod closing;
mod commands;
mod configs;
mod console;
mod diagnostics;
mod discover;
mod fold;
mod home;
mod instance;
mod kernels;
mod locales;
mod maintenance;
mod network;
mod packages;
mod preferences;
mod services;
mod setup;
mod tweaks;
mod units;
mod users;

use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use eframe::CreationContext;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use toolbox_core::Snapshot;
use toolbox_gui_kit::desktop::FontSpec;
use toolbox_gui_kit::toasts::Toasts;
use toolbox_gui_kit::{fonts, icons};
use toolbox_news::News;
use toolbox_system::{CallError, Facts, Status};

use self::preferences::Preferences;
pub use self::preferences::PreferencesSetup;
pub use self::setup::{FIXED_VERSION, Setup};
use crate::apply::ApplyHandle;
use crate::boot::Boot;
use crate::bootrepair::BootRepair;
use crate::catalogs::Catalogs;
use crate::clipboard::SystemClipboard;
use crate::command::{CommandBus, CommandReceiver, CommandSender};
use crate::configs::Configs;
use crate::console::Console;
use crate::desktop::DesktopLook;
use crate::desktop::icon_files::AppIcons;
use crate::diagnostics::Diagnostics;
use crate::instance::Arrivals;
use crate::launch::Launcher;
use crate::maintenance::Maintenance;
use crate::network::Network;
use crate::notifications::Notifier;
use crate::packages::Packages;
use crate::page::PageId;
use crate::pages::home::{HomeFacts, HomeProblem};
use crate::pages::kernels::view::Dkms;
use crate::pages::{PageContext, Pages, Plans, Refusals};
use crate::pictures::{FALLBACK_THEME, Pictures};
use crate::power::Power;
use crate::programs::Programs;
use crate::queue::Queue;
use crate::reader::{Watched, fresh};
use crate::settings::{MotionChoice, ThemeChoice};
use crate::share::ShareLink;
use crate::shell::{ChangesState, Palette};
use crate::snapshot::{Reading, SnapshotHandle};
use crate::units::{Units, UnitsHandle};
use crate::users::Users;
use crate::{profiling, shell};

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
    sender: CommandSender,
    receiver: CommandReceiver,
    palette: Palette,
    queue: Queue,
    changes_ui: ChangesState,
    /// What a shortcut changed this frame, for the shell to jump to.
    snap: shell::Snap,
    fold: fold::Fold,
    console: Console,
    apply: Option<ApplyHandle>,
    clipboard: Option<SystemClipboard>,
    notifier: Option<Arc<dyn Notifier>>,
    snapshot: Option<SnapshotHandle>,
    reading: Reading,
    /// The window had the focus last frame, so regaining it reads the
    /// system again.
    focused: bool,
    catalogs: Catalogs,
    desktops: Vec<String>,
    version: &'static str,
    dry_run: bool,
    refusals: Refusals,
    dkms: Watched<Dkms>,
    status: Watched<Status>,
    facts: Watched<Facts>,
    news: Watched<News>,
    programs: Watched<Programs>,
    power: Option<Arc<dyn Power>>,
    units: Option<UnitsHandle>,
    unit_lists: Units,
    launcher: Option<Arc<dyn Launcher>>,
    /// Refusals of a restart, sent from logind's task.
    restart_refused: (Sender<CallError>, Receiver<CallError>),
    home_problem: Option<HomeProblem>,
    maintenance: Maintenance,
    diagnostics: Diagnostics,
    boot: Boot,
    network: Network,
    packages: Packages,
    users: Users,
    configs: Configs,
    boot_repair: BootRepair,
    output_share: Option<ShareLink>,
    /// Told on exit, so no raise touches the closing window.
    arrivals: Option<Arc<Arrivals>>,
    plans: Plans,
    closing: closing::Closing,
    /// The probed snapshot plus the lock flags the Users page read.
    seen: Arc<Snapshot>,
    pages: Pages,
    /// The icon theme's pictures for the catalogs' apps.
    catalog_pictures: Option<Arc<AppIcons>>,
    pictures: Option<Pictures>,
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
        let CommandBus { sender, receiver } = setup.commands.unwrap_or_default();
        let mut maintenance = Maintenance::new(setup.maintenance, cc.egui_ctx.clone());
        maintenance.stock(setup.catalogs.skel.as_ref());
        let app = Self {
            maintenance,
            pages: preferences::pages_for(&prefs),
            prefs,
            state: restored(cc, setup.open),
            look: setup.look,
            fonts: (None, None, false),
            sender,
            receiver,
            palette: Palette::default(),
            queue: setup.queue,
            changes_ui: ChangesState::default(),
            snap: shell::Snap::default(),
            fold: fold::Fold::default(),
            console: Console::new(),
            apply: setup.apply,
            clipboard: setup.system_clipboard.then(|| SystemClipboard::of(cc)),
            notifier: setup.notifier,
            snapshot: setup.snapshot,
            reading: Reading::default(),
            focused: true,
            catalogs: setup.catalogs,
            desktops: setup.desktops,
            version: setup.version,
            dry_run: setup.dry_run,
            refusals: Refusals::default(),
            dkms: Watched::new(setup.dkms),
            status: Watched::new(setup.status),
            facts: Watched::new(setup.facts),
            news: Watched::new(setup.news),
            programs: Watched::new(setup.programs),
            power: setup.power,
            launcher: setup.launcher,
            units: setup.units,
            unit_lists: Units::default(),
            restart_refused: crossbeam_channel::unbounded(),
            home_problem: (setup.queue_lost > 0)
                .then_some(HomeProblem::QueueLost(setup.queue_lost)),
            diagnostics: Diagnostics::new(setup.diagnostics),
            boot: Boot::new(setup.boot, cc.egui_ctx.clone()),
            network: Network::new(setup.network, cc.egui_ctx.clone()),
            packages: Packages::new(setup.packages, cc.egui_ctx.clone()),
            users: Users::new(setup.users, cc.egui_ctx.clone()),
            configs: Configs::new(setup.configs, cc.egui_ctx.clone()),
            boot_repair: BootRepair::new(setup.boot_repair, cc.egui_ctx.clone()),
            output_share: setup.output_share,
            arrivals: setup.arrivals,
            plans: Plans::default(),
            closing: closing::Closing::default(),
            seen: Arc::default(),
            catalog_pictures: None,
            pictures: setup.pictures,
            toasts: Toasts::default(),
        };

        app.apply_language(&cc.egui_ctx);
        app.apply_palettes(&cc.egui_ctx);
        app.apply_choices(&cc.egui_ctx);
        app
    }

    fn take_look(&mut self, ctx: &egui::Context, desktop: &DesktopLook) {
        let look = &desktop.look;
        self.prefs.desktop = (look.theme, look.reduced_motion);
        self.prefs.palettes = (look.dark, look.light);
        self.apply_palettes(ctx);
        self.apply_choices(ctx);
        icons::use_theme(ctx, desktop.icons.clone());
        self.catalog_pictures.clone_from(&desktop.app_icons);
        if let Some(pictures) = &self.pictures {
            let theme = desktop
                .icons
                .as_ref()
                .map_or(FALLBACK_THEME, |icons| icons.theme());

            pictures.set_theme(theme);
        }

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
        let changes = self.queue.views();
        let held = self.apply_held();
        let snapshot = self.current_snapshot();
        let context = PageContext {
            snapshot: &snapshot,
            probed: self.reading.get().is_some(),
            changes: self.queue.changes(),
            revision: self.queue.revision(),
            catalogs: &self.catalogs,
            refusals: &self.refusals,
            dkms: self.dkms.reading(),
            busy: self.console.running(),
            console_open: self.state.console_open,
            units: &self.unit_lists,
            home: HomeFacts {
                status: self.status.fed(),
                facts: self.facts.fed(),
                news: self.news.fed(),
                recent: &self.state.recent,
                setup_finished: self.state.setup_finished,
                programs: self.programs.latest(),
                programs_failed: self.programs.reading().error().is_some(),
                problem: self.home_problem.as_ref(),
            },
            maintenance: &self.maintenance,
            diagnostics: &self.diagnostics,
            boot: &self.boot,
            network: &self.network,
            packages: &self.packages,
            users: &self.users,
            configs: &self.configs,
            boot_repair: &self.boot_repair,
            plans: self.plans,
            desktops: &self.desktops,
            version: self.version,
            app_icons: self.catalog_pictures.as_deref(),
            pictures: self.pictures.as_ref(),
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
        crate::instance::ready(arrivals, &cc.egui_ctx, cc, cc);
    }
}
