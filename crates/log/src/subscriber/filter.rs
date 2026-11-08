//! Which events reach the log file: the settings' level for our crates,
//! dependencies capped, or `GARUDA_TOOLBOX_LOG` in its place.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::fmt::Write as _;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

const OURS: [&str; 2] = ["toolbox_", "garuda_toolbox"];

#[derive(Debug, PartialEq, Eq)]
pub enum Origin {
    Settings,
    Override(String),
    Rejected { value: String, error: String },
}

/// Our crates at `level`. Dependencies stay at warn, so their chatter
/// never floods the log and their warnings are never lost, and open up to
/// info only for trace ("Everything"). Deeper takes `GARUDA_TOOLBOX_LOG`.
#[must_use = "the filter does nothing until installed"]
pub fn settings_filter(level: LevelFilter) -> EnvFilter {
    if level == LevelFilter::TRACE {
        return parsed(&directives(LevelFilter::INFO, level));
    }

    // The Vulkan loader warns at every start about each driver it skips,
    // which says nothing about this app.
    let quiet = format!("{},wgpu_hal=error", directives(LevelFilter::WARN, level));
    parsed(&quiet)
}

#[must_use = "the filter does nothing until installed"]
pub fn chosen(level: LevelFilter, variable: Option<&OsStr>) -> (EnvFilter, Origin) {
    let Some(variable) = variable.filter(|value| !value.is_empty()) else {
        return (settings_filter(level), Origin::Settings);
    };

    let value = variable.to_string_lossy().into_owned();
    let Some(text) = variable.to_str() else {
        let error = "it is not UTF-8".to_owned();
        return (settings_filter(level), Origin::Rejected { value, error });
    };

    match EnvFilter::builder().parse(text) {
        Ok(filter) => (filter, Origin::Override(value)),
        Err(error) => {
            let error = error.to_string();
            (settings_filter(level), Origin::Rejected { value, error })
        }
    }
}

#[must_use = "the filter does nothing until installed"]
pub fn profiler_filter() -> EnvFilter {
    parsed(&directives(LevelFilter::WARN, LevelFilter::TRACE))
}

pub fn describe(level: LevelFilter, origin: &Origin) -> String {
    match origin {
        Origin::Override(value) => format!("log filter {value}"),
        Origin::Settings | Origin::Rejected { value: _, error: _ } => {
            format!("log level {}", level.to_string().to_ascii_lowercase())
        }
    }
}

/// `dependencies,toolbox_=ours,garuda_toolbox=ours`. `EnvFilter`
/// matches a target by its start, so `toolbox_` covers every crate.
fn directives(dependencies: LevelFilter, ours: LevelFilter) -> String {
    let mut text = dependencies.to_string().to_ascii_lowercase();
    let ours = ours.to_string().to_ascii_lowercase();

    for target in OURS {
        write!(text, ",{target}={ours}").ok();
    }

    text
}

/// Our own directives always parse. Should one not, nothing is filtered
/// out rather than everything.
fn parsed(directives: &str) -> EnvFilter {
    EnvFilter::builder()
        .parse(directives)
        .unwrap_or_else(|_unparsable| EnvFilter::new("trace"))
}
