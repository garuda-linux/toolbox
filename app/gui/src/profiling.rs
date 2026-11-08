//! The Tracy profiler behind the `profiling` feature.
//!
//! Trace spans of the frame, the pages and the row loops reach Tracy only
//! with the feature. Without it no Tracy code is built and a span costs one
//! level check.
//!
//! Build with `--features profiling` and connect Tracy to the running app
//! (localhost only).

/// The layer that hands the spans to Tracy, for
/// `toolbox_log::subscriber::init`. Starts Tracy's client.
#[cfg(feature = "profiling")]
#[must_use = "the layer does nothing until installed"]
pub fn layer() -> toolbox_log::subscriber::Profiler {
    Box::new(tracing_tracy::TracyLayer::default())
}

/// Ends Tracy's frame, so the profiler shows one bar per egui frame.
#[cfg(feature = "profiling")]
pub fn frame_mark() {
    if let Some(client) = tracing_tracy::client::Client::running() {
        client.frame_mark();
    }
}

#[cfg(not(feature = "profiling"))]
pub const fn frame_mark() {}
