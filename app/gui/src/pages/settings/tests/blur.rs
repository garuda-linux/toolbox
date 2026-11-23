use egui::accesskit::{Role, Toggled};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_config::Settings;

use super::super::SettingsPage;
use crate::testing::{PageFixture, PageState};

const BLUR: &str = "Blur the desktop behind the window";

fn page(blur_desktop: bool, desktop_blur: bool) -> Harness<'static, PageState<SettingsPage>> {
    let mut settings = Settings::default();
    settings.window.blur_desktop = blur_desktop;

    let mut harness = PageFixture {
        settings,
        desktop_blur,
        ..PageFixture::default()
    }
    .harness(SettingsPage::default());

    harness.run();
    harness
}

/// The switch's state, and whether it takes a click.
fn switch(harness: &Harness<'static, PageState<SettingsPage>>) -> (Option<Toggled>, bool) {
    let node = harness.get_by_role_and_label(Role::CheckBox, BLUR);
    let node = node.accesskit_node();
    (node.toggled(), node.is_disabled())
}

#[test]
fn without_blur_on_the_desktop_the_switch_is_off_and_disabled_with_the_reason() {
    let harness = page(true, false);

    assert_eq!(switch(&harness), (Some(Toggled::False), true));
    assert!(
        harness
            .query_by_label_contains("does not offer blur")
            .is_some()
    );
}

#[test]
fn with_blur_on_the_desktop_the_switch_follows_the_setting() {
    for on in [true, false] {
        let harness = page(on, true);
        let toggled = if on { Toggled::True } else { Toggled::False };

        assert_eq!(switch(&harness), (Some(toggled), false), "setting {on}");
        assert!(
            harness
                .query_by_label_contains("does not offer blur")
                .is_none()
        );
    }
}
