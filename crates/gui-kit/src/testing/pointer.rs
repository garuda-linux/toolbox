//! Pointer input a test sends: clicks and the mouse wheel.

pub fn click_at<State>(harness: &mut egui_kittest::Harness<'_, State>, at: egui::Pos2) {
    let press = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };

    harness.event(egui::Event::PointerMoved(at));
    harness.event(press(true));
    harness.step();
    harness.event(press(false));
    harness.run();
}

/// Turns the wheel at `at`. A negative `delta` scrolls the content up.
pub fn wheel_at<State>(harness: &mut egui_kittest::Harness<'_, State>, at: egui::Pos2, delta: f32) {
    harness.event(egui::Event::PointerMoved(at));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, delta),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
    harness.run();
}

/// Turns the mouse wheel by `points` with the pointer at (200, 200) and
/// runs the harness until it settles.
pub fn scroll<State>(harness: &mut egui_kittest::Harness<'_, State>, points: f32) {
    harness.event(egui::Event::PointerMoved(egui::pos2(200.0, 200.0)));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -points),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
    harness.run();
}
