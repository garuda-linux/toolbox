//! Text with the parts a search matched washed in the accent, as Rerun's
//! filter widget marks them, so a hit is found at a glance.

use std::ops::Range;

use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Color32, FontId, Ui};

use crate::tokens;

/// `text` in `font` and `color`, every case-insensitive match of `query`
/// on the accent's wash. An empty query marks nothing.
#[must_use = "the job does nothing until laid out"]
pub fn highlighted(ui: &Ui, text: &str, query: &str, font: FontId, color: Color32) -> LayoutJob {
    // A wrapping line lays its first row as tall as the line and puts a
    // job's text at the bottom by default, below a title's middle.
    let plain = TextFormat {
        valign: Align::Center,
        ..TextFormat::simple(font, color)
    };

    let marked = TextFormat {
        background: tokens::current(ui.ctx()).accent_soft,
        ..plain.clone()
    };

    let mut job = LayoutJob::default();
    let mut done = 0;
    for range in matches(text, query) {
        job.append(
            text.get(done..range.start).unwrap_or_default(),
            0.0,
            plain.clone(),
        );
        job.append(
            text.get(range.clone()).unwrap_or_default(),
            0.0,
            marked.clone(),
        );
        done = range.end;
    }

    job.append(text.get(done..).unwrap_or_default(), 0.0, plain);
    job
}

/// The byte ranges of `text` where `query` matches, ignoring case and
/// never overlapping. Lowercase can change a character's length, so the
/// search runs on the lowercase text and maps each hit back.
pub fn matches(text: &str, query: &str) -> Vec<Range<usize>> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }

    let mut lower = String::with_capacity(text.len());
    let mut origin: Vec<Range<usize>> = Vec::with_capacity(text.len());
    for (start, character) in text.char_indices() {
        let end = start + character.len_utf8();
        for folded in character.to_lowercase() {
            lower.push(folded);
            origin.extend(std::iter::repeat_n(start..end, folded.len_utf8()));
        }
    }

    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = lower.get(from..).and_then(|rest| rest.find(&needle)) {
        let hit = from + offset;
        let last = hit + needle.len() - 1;
        if let (Some(first), Some(end)) = (origin.get(hit), origin.get(last)) {
            found.push(first.start..end.end);
        }

        from = hit + needle.len();
    }

    found
}
