//! The tracing layer behind the log lines: the span path, the message,
//! then the fields as `key=value`.

#[cfg(test)]
mod tests;

use std::fmt::{self, Write as _};
use std::time::{Duration, Instant};

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Subscriber};
use tracing_log::{AsLog as _, NormalizeEvent as _};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::{LookupSpan, SpanRef};

/// Hands each finished line (level, target, message) to `write`.
pub struct LineLayer<F> {
    write: F,
}

/// What a span keeps for the lines inside it.
struct SpanText {
    fields: String,
    opened: Instant,
}

/// The message and the `key=value` fields of one event or span.
#[derive(Default)]
struct Fields {
    message: String,
    pairs: String,
}

impl<F> LineLayer<F>
where
    F: Fn(log::Level, &str, &str) + Send + Sync + 'static,
{
    pub const fn new(write: F) -> Self {
        Self { write }
    }
}

impl<S, F> tracing_subscriber::Layer<S> for LineLayer<F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    F: Fn(log::Level, &str, &str) + Send + Sync + 'static,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut fields);

        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanText {
                fields: fields.pairs,
                opened: Instant::now(),
            });
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };

        let mut fields = Fields::default();
        values.record(&mut fields);

        if let Some(text) = span.extensions_mut().get_mut::<SpanText>() {
            join(&mut text.fields, &fields.pairs);
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // A `log` record arrives as an event of tracing-log's own
        // callsite. Its real target and level are in its fields.
        let normalized = event.normalized_metadata();
        let meta = normalized.as_ref().unwrap_or_else(|| event.metadata());

        let mut fields = Fields::default();
        event.record(&mut fields);

        let mut text = ctx
            .event_scope(event)
            .map(|scope| path(scope.from_root()))
            .unwrap_or_default();

        if !text.is_empty() {
            text.push_str(": ");
        }

        text.push_str(&fields.message);
        join(&mut text, &fields.pairs);

        (self.write)(meta.level().as_log(), meta.target(), &text);
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else {
            return;
        };

        // Trace spans time frames and rows for the profiler, a line each
        // would bury the log.
        let meta = span.metadata();
        if *meta.level() == Level::TRACE {
            return;
        }

        let elapsed = span
            .extensions()
            .get::<SpanText>()
            .map(|text| text.opened.elapsed());

        let text = format!(
            "{}: close elapsed={}",
            path(span.scope().from_root()),
            duration(elapsed.unwrap_or_default())
        );

        (self.write)(meta.level().as_log(), meta.target(), &text);
    }
}

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            value.clone_into(&mut self.message);
        } else {
            self.pair(field.name(), value);
        }
    }

    // tracing hands every value but strings over as Debug, a Display one
    // (`%value`) included.
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else {
            self.pair(field.name(), &format!("{value:?}"));
        }
    }
}

impl Fields {
    /// `key=value`, quoted when the value would not read as one word. The
    /// fields `tracing-log` adds (`log.target`, …) are left out.
    #[expect(clippy::use_debug, reason = "Debug of a str quotes and escapes it")]
    fn pair(&mut self, key: &str, value: &str) {
        if key.starts_with("log.") {
            return;
        }

        if !self.pairs.is_empty() {
            self.pairs.push(' ');
        }

        let plain = !value.is_empty()
            && !value
                .chars()
                .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '='));

        if plain {
            write!(self.pairs, "{key}={value}").ok();
        } else {
            write!(self.pairs, "{key}={value:?}").ok();
        }
    }
}

/// `outer{a=1}:inner`, the spans from the root down.
fn path<'a, R>(spans: impl Iterator<Item = SpanRef<'a, R>>) -> String
where
    R: LookupSpan<'a> + 'a,
{
    let mut out = String::new();

    for span in spans {
        if !out.is_empty() {
            out.push(':');
        }

        out.push_str(span.name());

        if let Some(text) = span.extensions().get::<SpanText>()
            && !text.fields.is_empty()
        {
            write!(out, "{{{}}}", text.fields).ok();
        }
    }

    out
}

/// `more` after `text`, a space between them when both have words.
fn join(text: &mut String, more: &str) {
    if more.is_empty() {
        return;
    }

    if !text.is_empty() {
        text.push(' ');
    }

    text.push_str(more);
}

fn duration(elapsed: Duration) -> String {
    if elapsed < Duration::from_secs(1) {
        format!("{:.1}ms", elapsed.as_secs_f64() * 1000.0)
    } else {
        format!("{:.3}s", elapsed.as_secs_f64())
    }
}
