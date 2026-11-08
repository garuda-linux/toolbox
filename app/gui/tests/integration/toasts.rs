//! A failed background read raises no note. Its page tells of it and
//! reads again from its banner.

use crossbeam_channel::{Receiver, Sender};
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_gui::pages::kernels::view::Dkms;
use toolbox_gui::testing::PATIENCE;
use toolbox_gui::{App, Setup, reader, snapshot};

use crate::common::{HEIGHT, no_note, window};
use crate::system::step_until;

const FAILED: &str = "dkms could not list its modules.";

type Answer = Result<Dkms, String>;

/// The app with a dkms reader that answers what the test sends.
fn app() -> (Harness<'static, App>, Runtime, Sender<Answer>) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let (answer, answers): (Sender<Answer>, Receiver<Answer>) = crossbeam_channel::unbounded();

    let harness = window(HEIGHT).build_eframe(move |cc| {
        let read = move || answers.recv_timeout(PATIENCE).unwrap();
        let dkms = reader::spawn_fallible(
            &handle,
            toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
            "dkms",
            read,
        );

        let probe = || (crate::kernels::system(true), Vec::new());

        let setup = Setup {
            dkms: Some(dkms),
            snapshot: Some(snapshot::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                probe,
            )),
            ..Setup::default()
        };

        App::new(cc, setup)
    });

    (harness, runtime, answer)
}

#[test]
fn a_failed_read_shows_on_its_page_with_read_again_and_raises_no_note() {
    let (mut harness, _runtime, answer) = app();
    harness.get_by_role_and_label(Role::Tab, "Kernels").click();

    answer.send(Err("dkms: not found".to_owned())).unwrap();
    step_until(&mut harness, "the dkms failure", |harness| {
        harness.query_by_label(FAILED).is_some()
    });

    assert!(no_note(&harness));

    harness
        .query_all_by_label("Read again")
        .max_by(|one, other| one.rect().top().total_cmp(&other.rect().top()))
        .unwrap()
        .click();
    answer.send(Ok(Dkms::NotInstalled)).unwrap();
    step_until(&mut harness, "the failure gone", |harness| {
        harness.query_by_label(FAILED).is_none()
    });
}
