//! The app's tracing subscriber: the rotating file and stderr lines with
//! the span path and the fields. The `log` records of egui, wgpu and the
//! rest come in through tracing-log's `LogTracer`.

mod filter;
mod layer;
#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::sync::OnceLock;

use parking_lot::Mutex;
use tracing::Subscriber;
use tracing::level_filters::LevelFilter;
use tracing_log::{AsLog as _, AsTrace as _, LogTracer};
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::layer::{Layered, SubscriberExt as _};
use tracing_subscriber::{EnvFilter, Layer as _, Registry, reload};

use self::filter::{Origin, chosen, describe, profiler_filter, settings_filter};
use self::layer::LineLayer;
use crate::file::FileLogger;

/// The variable that replaces the settings' filter, in `EnvFilter`
/// syntax (`zbus=trace,toolbox_gui=debug`).
pub const ENV: &str = "GARUDA_TOOLBOX_LOG";

static ACTIVE: OnceLock<Reload> = OnceLock::new();

pub type Profiler = Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync>;

type Inner = Layered<Option<Filtered<Profiler, EnvFilter, Registry>>, Registry>;

struct Stack {
    subscriber: Box<dyn Subscriber + Send + Sync>,
    reload: Reload,
}

struct Reload {
    handle: reload::Handle<EnvFilter, Inner>,
    origin: Origin,
    level: Mutex<LevelFilter>,
    log_level: Mutex<log::LevelFilter>,
}

impl Reload {
    /// Our crates at `level` from now on. Does nothing while
    /// `GARUDA_TOOLBOX_LOG` decides, and says so.
    fn set(&self, level: LevelFilter) -> bool {
        *self.level.lock() = level;

        if matches!(self.origin, Origin::Override(_)) {
            return false;
        }

        let filter = settings_filter(level);
        *self.log_level.lock() = log_level(&filter);

        // Fails only once the subscriber is gone, when nothing logs.
        self.handle.reload(filter).is_ok()
    }

    fn describe(&self) -> String {
        describe(*self.level.lock(), &self.origin)
    }

    fn log_level(&self) -> log::LevelFilter {
        *self.log_level.lock()
    }
}

/// `variable` is the value of [`ENV`]. Lines are buffered until
/// [`crate::open_file`]. Call once, first thing: a second call changes
/// nothing, and after another `log` logger the dependencies' records miss
/// the file.
pub fn init(level: LevelFilter, variable: Option<&OsStr>, profiler: Option<Profiler>) {
    let logger: &'static FileLogger = crate::LOGGER.get_or_init(FileLogger::buffered);
    let Stack { subscriber, reload } =
        build(level, variable, profiler, |level, target, message| {
            logger.emit(level, target, message);
        });

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        return;
    }

    let log_level = reload.log_level();
    if let Origin::Rejected { value, error } = &reload.origin {
        tracing::warn!("{ENV}={value} is no log filter ({error}), the settings' level is used");
    }

    ACTIVE.set(reload).ok();

    if LogTracer::init_with_filter(log_level).is_err() {
        tracing::warn!("another `log` logger runs, the dependencies' records miss the log");
    }
}

/// Our crates at `level` while running, as the settings ask. With
/// `GARUDA_TOOLBOX_LOG` set the variable keeps deciding.
pub fn set_level(level: LevelFilter) {
    let Some(reload) = ACTIVE.get() else {
        return;
    };

    if reload.set(level) {
        log::set_max_level(reload.log_level());
    }
}

/// What the start line says about the filter: the subscriber's own words
/// once [`init`] ran, else the `log` level.
pub(crate) fn description() -> String {
    ACTIVE.get().map_or_else(
        || describe(log::max_level().as_trace(), &Origin::Settings),
        Reload::describe,
    )
}

fn build<F>(
    level: LevelFilter,
    variable: Option<&OsStr>,
    profiler: Option<Profiler>,
    write: F,
) -> Stack
where
    F: Fn(log::Level, &str, &str) + Send + Sync + 'static,
{
    let (filter, origin) = chosen(level, variable);
    let log_level = log_level(&filter);
    let (filter, handle) = reload::Layer::new(filter);

    let subscriber = Registry::default()
        .with(profiler.map(|layer| layer.with_filter(profiler_filter())))
        .with(LineLayer::new(write).with_filter(filter));

    Stack {
        subscriber: Box::new(subscriber),
        reload: Reload {
            handle,
            origin,
            level: Mutex::new(level),
            log_level: Mutex::new(log_level),
        },
    }
}

/// The most a `log` record can pass: records above it are dropped by
/// the `log` macros already, before any filter runs.
fn log_level(filter: &EnvFilter) -> log::LevelFilter {
    filter
        .max_level_hint()
        .map_or(log::LevelFilter::Trace, |hint| hint.as_log())
}
