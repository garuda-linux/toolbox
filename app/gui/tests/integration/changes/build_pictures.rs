//! Pictures of the queued AUR builds in the popover, in both themes.

use std::sync::Arc;

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use tokio::runtime::Runtime;
use toolbox_gui::apply::{ApplyHandle, PlanId, Report};
use toolbox_gui::queue::Queue;
use toolbox_gui::waking::Wake;
use toolbox_gui::{App, Setup};
use toolbox_plan::StepId;
use toolbox_plan::protocol::Event;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::testing::aur::order::{MISSING_STDOUT, OCTOPI_GIT_STDOUT};
use toolbox_system::testing::aur_orders::FakeAurOrders;

use super::build_order::{queued_build, with_orders_in};
use super::{queued, review};
use crate::common::{HEIGHT, installing, launch_in, window};
use crate::system::step_until;

const THEMES: [(Theme, &str); 2] = [(Theme::Dark, "dark"), (Theme::Light, "light")];

const REVIEW: &str = "paru -G -- octopi-git in ~/.cache/garuda-toolbox/aur-clone, then show the build files and ask before any build";

#[test]
fn the_built_row_with_every_package_it_pulls_in() {
    let runtime = Runtime::new().unwrap();
    let mut results = SnapshotResults::new();

    for (theme, suffix) in THEMES {
        let orders = Arc::new(FakeAurOrders::new(vec![(
            "octopi-git",
            Ok(OCTOPI_GIT_STDOUT),
        )]));

        let mut harness = with_orders_in(
            window(HEIGHT).with_theme(theme),
            &runtime,
            queued_build("octopi-git", AurRoute::Accepted),
            orders,
        );

        harness.run();
        review(&mut harness);
        step_until(&mut harness, "the build", |harness| {
            harness
                .query_by_label_contains("Builds alpm_octopi_utils-git, octopi-git.")
                .is_some()
        });

        harness
            .get_by_role_and_label(Role::Button, "Packages 60")
            .click();
        harness.run();
        results.add(harness.try_snapshot(format!("changes/build_ready_{suffix}")));
    }
}

#[test]
fn the_hold_names_the_build_and_offers_read_again() {
    let runtime = Runtime::new().unwrap();
    let mut results = SnapshotResults::new();

    for (theme, suffix) in THEMES {
        let orders = Arc::new(FakeAurOrders::new(vec![(
            "no-such-package-toolbox",
            Ok(MISSING_STDOUT),
        )]));

        let mut harness = with_orders_in(
            window(HEIGHT).with_theme(theme),
            &runtime,
            queued_build("no-such-package-toolbox", AurRoute::Accepted),
            orders,
        );

        harness.run();
        review(&mut harness);
        step_until(&mut harness, "the reason", |harness| {
            harness
                .query_all_by_label_contains("has no build order: the AUR does not have it.")
                .next()
                .is_some()
        });

        results.add(harness.try_snapshot(format!("changes/build_hold_{suffix}")));
    }
}

#[test]
fn the_old_path_row_says_why_paru_asks_again() {
    let mut results = SnapshotResults::new();

    for (theme, suffix) in THEMES {
        let changes = queued_build(
            "yay-bin",
            AurRoute::Refused {
                kind: RouteKind::Shared,
                detail: String::new(),
            },
        );

        let mut harness: Harness<'static, App> =
            launch_in(window(HEIGHT).with_theme(theme), queued(changes));

        harness.run();
        review(&mut harness);

        results.add(harness.try_snapshot(format!("changes/build_old_path_{suffix}")));
    }
}

/// The review of a build waits in the Console, with the hint and Continue.
#[test]
fn the_console_review_offers_continue_beside_its_hint() {
    let mut results = SnapshotResults::new();

    for (theme, suffix) in THEMES {
        let (reports, apply) = ApplyHandle::feeding(Wake::none());
        let mut harness = window(HEIGHT).with_theme(theme).build_eframe(move |cc| {
            App::new(
                cc,
                Setup {
                    apply: Some(apply),
                    queue: Queue::in_memory(installing(&["firefox"])),
                    ..Setup::default()
                },
            )
        });

        harness.run();
        harness.get_by_label("Apply").click();
        harness.run();

        reports.send(Report::PlanStarted {
            plan: PlanId::new(1),
            steps: vec![(StepId::new(1), vec![REVIEW.to_owned()])],
        });
        reports.send(Report::Step(Event::Started {
            step: StepId::new(1),
        }));
        harness.run();

        assert!(harness.query_by_label("Continue").is_some());
        results.add(harness.try_snapshot(format!("changes/console_review_{suffix}")));
    }
}
