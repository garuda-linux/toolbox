use egui::text::LayoutJob;
use egui::{Color32, Context};

use super::{CodeJob, Ink};
use crate::tokens;

fn ink_of(job: &LayoutJob, part: &str) -> Option<Color32> {
    let start = job.text.find(part)?;
    job.sections
        .iter()
        .find(|section| section.byte_range.start.0 == start)
        .map(|section| section.format.color)
}

fn built(fill: impl Fn(&mut CodeJob, &egui::Ui)) -> (LayoutJob, tokens::CodeInks) {
    let ctx = Context::default();
    crate::install(&ctx);
    let mut out = None;
    let output = ctx.run_ui(egui::RawInput::default(), |ui| {
        let mut job = CodeJob::new(ui);
        fill(&mut job, ui);
        out = Some(job.finish());
    });

    output.drop_without_applying_deltas();
    let job = out.unwrap_or_default();
    (job, tokens::current(&ctx).code)
}

#[test]
fn json_keys_and_values_take_the_terminal_inks() {
    let text = "{\"on\": true, \"days\": 3}";
    let (job, inks) = built(|job, ui| job.highlighted(ui, text, "json"));

    assert_eq!(job.text, text, "the same text to select and hear");
    assert_eq!(ink_of(&job, "\"on\""), Some(inks.string));
    assert_eq!(ink_of(&job, "true"), Some(inks.keyword));
    assert_eq!(ink_of(&job, "3"), Some(inks.plain));
}

#[test]
fn each_ink_takes_its_terminal_colour() {
    let (job, inks) = built(|job, _| {
        job.inked("$ ", Ink::Dim);
        job.inked("garuda-toolbox", Ink::Function);
        job.plain(" key=");
        job.inked("value", Ink::String);
        job.inked(" --force", Ink::Keyword);
    });

    assert_eq!(job.text, "$ garuda-toolbox key=value --force");
    assert_eq!(ink_of(&job, "$ "), Some(inks.dim));
    assert_eq!(ink_of(&job, "garuda"), Some(inks.function));
    assert_eq!(ink_of(&job, " key"), Some(inks.plain));
    assert_eq!(ink_of(&job, "value"), Some(inks.string));
    assert_eq!(ink_of(&job, " --force"), Some(inks.keyword));
}
