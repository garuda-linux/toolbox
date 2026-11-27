//! A group whose items share one Tab stop. Only the stop is focusable, the
//! others only clickable, and the arrow keys, Home and End move the stop
//! and the focus with it. The stop lives in egui's temp data under the
//! group's id, and an item id is derived from the item's index or key, so
//! the group can focus a neighbour it has not drawn yet.

use egui::style::ScrollAnimation;
use egui::{
    EventFilter, FocusDirection, Id, InputState, Key, Modifiers, Popup, Response, Sense, Ui,
};

/// Which arrow keys move the stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Axes {
    /// Up and Down, for the rows of a list.
    Vertical,
    /// All four arrows, for a line of chips.
    Both,
}

/// How a group reads its keys and where its stop rests.
#[derive(Clone, Copy, Debug)]
pub(super) struct Setup {
    /// The stop while no item holds the focus. `None` keeps the stop where
    /// the focus left it, and a group without a stop yet starts at the
    /// first item.
    pub rest: Option<usize>,
    pub axes: Axes,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct State {
    stop: usize,
    /// The item to focus once it is drawn as the stop, since an item that
    /// was not focusable this frame cannot take the focus yet.
    pending: Option<usize>,
}

/// One frame of a group, read before its items are drawn.
#[derive(Clone, Copy, Debug)]
pub(super) struct Roving {
    group: Id,
    axes: Axes,
    stop: usize,
    pending: Option<usize>,
    held: Option<usize>,
    moved: bool,
}

impl Roving {
    /// Reads the group's stop and moves it on this frame's keys. The keys
    /// count only while the stop holds the focus, so a group the focus
    /// left keeps its arrows for the rest of the page.
    pub fn read(
        ui: &Ui,
        group: Id,
        count: usize,
        item: impl Fn(usize) -> Id,
        setup: Setup,
    ) -> Self {
        let kept = ui.data(|data| data.get_temp::<State>(group));
        let stop = kept
            .map_or_else(|| setup.rest.unwrap_or(0), |kept| kept.stop)
            .min(count.saturating_sub(1));

        let pending = kept
            .and_then(|kept| kept.pending)
            .filter(|&index| index < count);

        let focused = ui.memory(egui::Memory::focused);
        let on_stop = count > 0 && focused == Some(item(stop));
        let mut state = State { stop, pending };
        if pending.is_none() && !on_stop {
            let found = focus_index(ui, group, count, &item, focused);
            state.stop = found.or(setup.rest).unwrap_or(stop);
        }

        let mut held = None;
        let mut moved = false;
        let active = on_stop && !Popup::is_any_open(ui.ctx());
        if active {
            held = Some(stop);
            let target = keys(ui, setup.axes).map(|step| step.apply(stop, count));
            if let Some(target) = target.filter(|&target| target != stop) {
                state = State {
                    stop: target,
                    pending: Some(target),
                };
                held = None;
                moved = true;
            }
        }

        ui.data_mut(|data| data.insert_temp(group, state));
        Self {
            group,
            axes: setup.axes,
            stop: state.stop,
            pending: state.pending,
            held,
            moved,
        }
    }

    /// Whether this frame's keys moved the stop, so the item it now names
    /// was reached by the keyboard.
    pub const fn moved(&self) -> bool {
        self.moved
    }

    /// Whether item `index` is the Tab stop, the only focusable item.
    pub(super) const fn is_stop(&self, index: usize) -> bool {
        self.stop == index
    }

    /// The index of the Tab stop.
    pub const fn stop(&self) -> usize {
        self.stop
    }

    /// The stop's index while it holds the focus and no key moved it this
    /// frame, so a key of the frame belongs to that item alone.
    pub const fn held(&self) -> Option<usize> {
        self.held
    }

    /// Moves the stop to `index` and takes the focus there once it is drawn,
    /// as a page asks for a row it reveals.
    pub fn focus(&mut self, ui: &Ui, index: usize) {
        self.stop = index;
        self.pending = Some(index);
        self.held = None;
        let state = State {
            stop: index,
            pending: Some(index),
        };

        ui.data_mut(|data| data.insert_temp(self.group, state));
    }

    /// Takes the focus for a pending move and the stop for a click on an
    /// item that is not the stop. Call it once per item, after the item's
    /// response exists.
    pub fn apply(&self, ui: &Ui, index: usize, item: Id, response: &Response) {
        // Items answer in turn, so each one reads the state the last one left.
        let mut state = ui
            .data(|data| data.get_temp::<State>(self.group))
            .unwrap_or(State {
                stop: self.stop,
                pending: self.pending,
            });

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
                horizontal_arrows: self.axes == Axes::Both,
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

/// The index of the item that holds the focus, if it is in the group. The
/// stop is the one the focus is on in the usual case, so a search only
/// runs when the items moved under the focus, and its answer is kept for
/// this focus and item count.
fn focus_index(
    ui: &Ui,
    group: Id,
    count: usize,
    item: &impl Fn(usize) -> Id,
    focused: Option<Id>,
) -> Option<usize> {
    let focus = focused?;
    let kept_id = group.with("searched");
    let kept = ui.data(|data| data.get_temp::<(Id, usize, Option<usize>)>(kept_id));
    if let Some((was, was_count, index)) = kept
        && was == focus
        && was_count == count
    {
        return index;
    }

    let index = (0..count).find(|&index| item(index) == focus);
    ui.data_mut(|data| data.insert_temp(kept_id, (focus, count, index)));
    index
}

/// This frame's step, consumed so egui's own focus search does not act on
/// the same keys.
fn keys(ui: &Ui, axes: Axes) -> Option<Step> {
    let (step, arrow) = ui.input_mut(|input| read_keys(input, axes));
    if arrow {
        // egui's search for the widget in the arrow's direction would land
        // beside the stop on the same press.
        ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
    }

    step
}

/// The step and whether an arrow was pressed. Every arrow of the frame is
/// consumed, so a second one does not count as a step of its own.
fn read_keys(input: &mut InputState, axes: Axes) -> (Option<Step>, bool) {
    let plain = Modifiers::NONE;
    if input.consume_key(plain, Key::Home) {
        return (Some(Step::First), false);
    }

    if input.consume_key(plain, Key::End) {
        return (Some(Step::Last), false);
    }

    let (before_keys, after_keys): (&[Key], &[Key]) = match axes {
        Axes::Vertical => (&[Key::ArrowUp], &[Key::ArrowDown]),
        Axes::Both => (
            &[Key::ArrowUp, Key::ArrowLeft],
            &[Key::ArrowDown, Key::ArrowRight],
        ),
    };

    let before = before_keys
        .iter()
        .filter(|&&key| input.consume_key(plain, key))
        .count()
        > 0;

    let after = after_keys
        .iter()
        .filter(|&&key| input.consume_key(plain, key))
        .count()
        > 0;

    let step = match (before, after) {
        (true, false) => Some(Step::Previous),
        (false, true) => Some(Step::Next),
        _ => None,
    };

    (step, before || after)
}

/// The Sense of an item: the stop takes the focus, the others only the
/// click, so Tab reaches one item of the group.
pub(super) fn sense(is_stop: bool) -> Sense {
    if is_stop {
        Sense::click()
    } else {
        Sense::CLICK
    }
}
