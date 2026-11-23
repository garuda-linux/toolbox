//! The pictures of the user guide in `docs/src/images`, rendered from the
//! page fixtures in both themes. A run fails on a committed picture that
//! differs from a fresh render. `UPDATE_GUIDE_IMAGES=1` writes them
//! instead, after a wanted change.

mod captures;
mod files;

use egui::Theme;

use crate::look;

#[test]
fn every_guide_picture_matches_its_render() {
    let update = files::updating();
    let mut stale = Vec::new();
    for capture in &captures::ALL {
        for theme in [Theme::Dark, Theme::Light] {
            let picture = look::pictured(|| (capture.render)(theme));
            if let Err(problem) = files::settle(capture.name, theme, &picture, update) {
                stale.push(problem);
            }
        }
    }

    assert!(
        stale.is_empty(),
        "guide pictures differ, run with UPDATE_GUIDE_IMAGES=1 after a wanted change:\n{}",
        stale.join("\n")
    );
}
