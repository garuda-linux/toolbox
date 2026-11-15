//! A group whose items share one Tab stop. Only the current item is
//! focusable, the others only clickable, and the arrow keys, Home and End
//! move the stop and the focus with it. The stop lives in egui's temp
//! data under the group's id, and an item id is the group's id with the
//! item's index, so the group can focus a neighbour it has not drawn yet.

use egui::style::ScrollAnimation;
use egui::{
    EventFilter, FocusDirection, Id, InputState, Key, Modifiers, Popup, Response, Sense, Ui,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct State {
    stop: usize,
    /// The item to focus once it is drawn as the stop, since an item that
    /// was not focusable this frame cannot take the focus yet.
    pending: Option<usize>,
}

/// One frame of a group, read before its items are drawn.
#[derive(Clone, Copy, Debug)]
pub struct Roving {
    group: Id,
    state: State,
}

impl Roving {
    /// Reads the group's stop and moves it on this frame's keys. The keys
    /// count only while the stop holds the focus, so a group the focus
    /// left keeps its arrows for the rest of the page, and an open menu
    /// keeps them for its own items.
    pub fn read(ui: &Ui, group: Id, count: usize, item: impl Fn(usize) -> Id) -> Self {
        let kept = ui
            .data(|data| data.get_temp::<State>(group))
            .unwrap_or_default();

        let stop = kept.stop.min(count.saturating_sub(1));
        let mut state = State {
            stop,
            pending: kept.pending.filter(|&index| index < count),
        };

        let focused = ui.memory(egui::Memory::focused) == Some(item(stop));
        if count > 0 && focused && !Popup::is_any_open(ui.ctx()) {
            let target = keys(ui).map(|step| step.apply(stop, count));
            if let Some(target) = target.filter(|&target| target != stop) {
                state = State {
                    stop: target,
                    pending: Some(target),
                };
            }
        }

        ui.data_mut(|data| data.insert_temp(group, state));
        Self { group, state }
    }

    /// Whether item `index` is the Tab stop, the only focusable item.
    pub const fn is_stop(&self, index: usize) -> bool {
        self.state.stop == index
    }

    /// Takes the focus for a pending move and the stop for a click on an
    /// item that is not the stop. Call it once per item, after the item's
    /// response exists.
    pub fn apply(&self, ui: &Ui, index: usize, item: Id, response: &Response) {
        // Items answer in turn, so each one reads the state the last one left.
        let mut state = ui
            .data(|data| data.get_temp::<State>(self.group))
            .unwrap_or(self.state);

        if state.pending == Some(index) && state.stop == index {
            ui.memory_mut(|memory| memory.request_focus(item));
            response.scroll_to_me_animation(None, ScrollAnimation::none());
            state.pending = None;
        }

        if response.clicked() && state.stop != index {
            state = State {
                stop: index,
                pending: Some(index),
            };
        }

        if response.has_focus() {
            // The group keeps the arrows it reads, so egui's own search does
            // not move the focus to a control beside the stop.
            let filter = EventFilter {
                horizontal_arrows: true,
                vertical_arrows: true,
                ..EventFilter::default()
            };

            ui.memory_mut(|memory| memory.set_focus_lock_filter(item, filter));
        }

        ui.data_mut(|data| data.insert_temp(self.group, state));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Previous,
    Next,
    First,
    Last,
}

impl Step {
    fn apply(self, stop: usize, count: usize) -> usize {
        match self {
            Self::Previous => stop.saturating_sub(1),
            Self::Next => (stop + 1).min(count - 1),
            Self::First => 0,
            Self::Last => count - 1,
        }
    }
}

/// This frame's step, consumed so egui's own focus search does not act on
/// the same keys.
fn keys(ui: &Ui) -> Option<Step> {
    let (step, arrow) = ui.input_mut(read_keys);
    if arrow {
        // egui's search for the widget in the arrow's direction would land
        // beside the stop on the same press.
        ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
    }

    step
}

/// The step and whether an arrow was pressed. Every arrow of the frame is
/// consumed, so a second one does not count as a step of its own.
fn read_keys(input: &mut InputState) -> (Option<Step>, bool) {
    let plain = Modifiers::NONE;
    if input.consume_key(plain, Key::Home) {
        return (Some(Step::First), false);
    }

    if input.consume_key(plain, Key::End) {
        return (Some(Step::Last), false);
    }

    let before = [Key::ArrowUp, Key::ArrowLeft]
        .map(|key| input.consume_key(plain, key))
        .into_iter()
        .any(|hit| hit);

    let after = [Key::ArrowDown, Key::ArrowRight]
        .map(|key| input.consume_key(plain, key))
        .into_iter()
        .any(|hit| hit);

    let step = match (before, after) {
        (true, false) => Some(Step::Previous),
        (false, true) => Some(Step::Next),
        _ => None,
    };

    (step, before || after)
}

/// The Sense of an item: the stop takes the focus, the others only the
/// click, so Tab reaches one item of the group.
pub fn sense(is_stop: bool) -> Sense {
    if is_stop {
        Sense::click()
    } else {
        Sense::CLICK
    }
}
