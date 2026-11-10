//! `status --watch`: the facts again and again, printed only when they
//! changed since the last time, each under the time they were read.

#[cfg(test)]
mod tests;

use std::io::Write as _;

use toolbox_cli::Exit;
use toolbox_cli::output::{Paint, json_line};

use super::{StatusView, brief, text};
use crate::out::Out;

pub enum Shape {
    Text { paint: Paint, verbose: bool },
    Brief,
    Json,
}

pub struct Watch {
    shape: Shape,
    last: Option<StatusView>,
}

impl Watch {
    pub const fn new(shape: Shape) -> Self {
        Self { shape, last: None }
    }

    /// What `view`, read at `time`, adds: `None` when it is what was
    /// printed last.
    pub fn next(&mut self, view: StatusView, time: &str) -> Option<String> {
        if self.last.as_ref() == Some(&view) {
            return None;
        }

        let apart = if self.last.is_some() { "\n" } else { "" };
        let shown = match &self.shape {
            Shape::Text { paint, verbose } => {
                let block = text(&view, *paint, *verbose);
                format!("{apart}{}\n{block}", paint.dim(time))
            }
            Shape::Brief => format!("{time}  {}\n", brief(&view)),
            Shape::Json => json_line(&view),
        };

        self.last = Some(view);
        Some(shown)
    }
}

/// Reads with `read` and prints each change, until `wait` says a Ctrl+C
/// or a signal came. That is how a watch is meant to end, so it ends
/// with 0, not with the signal's code.
pub fn watch(
    mut watching: Watch,
    mut read: impl FnMut() -> StatusView,
    mut stopped: impl FnMut() -> bool,
    clock: impl Fn() -> String,
    out: &mut Out<'_>,
) -> Exit {
    loop {
        let view = read();
        if let Some(shown) = watching.next(view, &clock()) {
            out.write_all(shown.as_bytes()).ok();
            out.flush().ok();
        }

        if stopped() {
            return Exit::Success;
        }
    }
}
