use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::diagnostics::{Known, redact};
use toolbox_services::share::Upload;

use super::spawn;
use super::view::{Question, ShareView, Subject, ask};
use crate::command::{AppCommand, Command};
use crate::diagnostics::DryRunTransport;

#[test]
fn a_share_runs_once_at_a_time_and_ends_with_the_link() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let link = spawn(
        runtime.handle(),
        crate::waking::Wake::none(),
        Arc::new(DryRunTransport),
        Known::default(),
    );

    let mut upload = Upload::default();

    assert!(
        !link.share(&mut upload, String::new()),
        "an empty text went out"
    );
    assert!(link.share(&mut upload, "the output".to_owned()));
    assert_eq!(upload, Upload::Running);
    assert!(!link.share(&mut upload, "again".to_owned()));

    let start = Instant::now();
    while !link.take(&mut upload) {
        assert!(start.elapsed() < Duration::from_secs(10), "no link came");
        std::thread::sleep(Duration::from_millis(5));
    }

    let shared = upload.paste().map(|paste| paste.link.as_str());
    assert!(
        shared
            .is_some_and(|text| text.starts_with("https://bin.garudalinux.org/?dryrun0000000000#")),
        "{upload:?}"
    );
}

/// The upload question over `text`, open while the state holds it.
fn question(text: &str) -> egui_kittest::Harness<'static, Option<Question>> {
    let shareable = redact(text, &Known::default());
    let (commands, _receiver) = crate::command::channel();
    let upload = Upload::default();
    let shown = move |ui: &mut egui::Ui, asking: &mut Option<Question>| {
        let view = ShareView {
            upload: &upload,
            subject: Subject::Report,
            commands: &commands,
            hide: Command::App(AppCommand::ReadSystem),
        };

        ask(ui.ctx(), &view, &shareable, asking);
    };

    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(900.0, 700.0))
        .with_context_setup(toolbox_gui_kit::install)
        .build_ui_state(shown, Some(Question::Upload));

    harness.run();
    harness
}

#[test]
fn the_next_question_starts_with_a_blank_search() {
    let mut harness = question("kernel: oops\n");
    harness.get_by_label("Search the text").click();
    harness.run();
    harness.get_by_label("Search the text").type_text("oops");
    harness.run();

    harness
        .get_by_role_and_label(Role::Button, "Cancel")
        .click();
    harness.run();
    assert_eq!(*harness.state(), None);

    *harness.state_mut() = Some(Question::Upload);
    harness.run();

    let typed = harness.get_by_label("Search the text").value();
    assert_eq!(typed.as_deref().unwrap_or_default(), "");
}

#[test]
fn the_search_marks_and_counts_a_match_in_a_long_text() {
    let lines: Vec<String> = (0_u32..5000).map(|line| format!("line {line}\n")).collect();
    let mut harness = question(&(lines.concat() + "the oops at the end\n"));

    harness.get_by_label("Search the text").click();
    harness.run();
    harness.get_by_label("Search the text").type_text("oops");
    harness.run();

    assert!(harness.query_by_label("1 of 1").is_some());
    assert!(
        harness
            .query_by_label_contains("the oops at the end")
            .is_some(),
        "the match was not brought into view"
    );
}

#[test]
fn the_link_menu_comes_after_hide() {
    let (commands, _receiver) = crate::command::channel();
    let upload = Upload::Done(toolbox_privatebin::Paste {
        link: "https://bin.garudalinux.org/?abc#key".to_owned(),
        delete_link: "https://bin.garudalinux.org/?pasteid=abc&deletetoken=t".to_owned(),
    });

    let shown = move |ui: &mut egui::Ui| {
        let view = ShareView {
            upload: &upload,
            subject: Subject::Report,
            commands: &commands,
            hide: Command::App(AppCommand::ReadSystem),
        };

        super::view::row(ui, &view, &mut None);
    };

    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(700.0, 100.0))
        .with_context_setup(toolbox_gui_kit::install)
        .build_ui(shown);

    harness.run();

    let hide = harness.get_by_role_and_label(Role::Button, "Hide").rect();
    let menu = harness.get_by_label("More link actions").rect();
    assert!(
        menu.left() > hide.right(),
        "the menu at {menu:?} is not after Hide at {hide:?}"
    );
}
