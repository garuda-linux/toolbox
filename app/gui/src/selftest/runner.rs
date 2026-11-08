//! Runs a suite inside the window, as an egui plugin: after each frame it
//! reads the AccessKit tree egui built and decides the step, before the
//! next frame it hands egui the pointer and key events of that step.

use std::collections::VecDeque;
use std::fmt::{self, Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use egui::accesskit::{Action, ActionRequest, NodeId, Role, TreeId};
use egui::{
    ColorImage, Context, Event, FullOutput, Modifiers, Plugin, RawInput, ScreenshotCallback,
    ViewportCommand,
};
use kittest::{AccessKitNode, NodeT, Queryable as _, State};

use super::step::Step;

pub const RESULT_FILE: &str = "result";

/// How long a shot waits after the step before it, longer than any slide
/// or fade of the kit, so no control is caught halfway.
const SETTLE: Duration = Duration::from_millis(400);

pub struct Runner {
    steps: VecDeque<Step>,
    number: usize,
    tree: Option<State>,
    events: Vec<Event>,
    started: Instant,
    patience: Duration,
    dir: PathBuf,
    shot: Option<Receiver<Arc<ColorImage>>>,
    /// The verdict of a failed step, written once the shot of the window
    /// at the failure is saved.
    failed: Option<String>,
    finished: bool,
}

struct Target {
    node: NodeId,
    tree: TreeId,
    disabled: bool,
}

#[derive(Clone, Copy)]
struct Found<'tree>(AccessKitNode<'tree>);

enum Outcome {
    Done,
    Waiting,
    Failed(String),
}

impl Debug for Found<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        kittest::debug_fmt_node(self, f)
    }
}

impl<'tree> NodeT<'tree> for Found<'tree> {
    fn accesskit_node(&self) -> AccessKitNode<'tree> {
        self.0
    }

    fn new_related(&self, child_node: AccessKitNode<'tree>) -> Self {
        Self(child_node)
    }
}

impl Runner {
    /// `dir` takes the screenshots and the verdict. A step that waits
    /// longer than `patience` fails the suite.
    #[must_use = "the runner does nothing until added as a plugin"]
    pub fn new(steps: Vec<Step>, dir: PathBuf, patience: Duration) -> Self {
        Self {
            steps: steps.into(),
            number: 1,
            tree: None,
            events: Vec::new(),
            started: Instant::now(),
            patience,
            dir,
            shot: None,
            failed: None,
            finished: false,
        }
    }

    fn advance(&mut self, ctx: &Context) {
        if let Some(verdict) = self.failed.clone() {
            self.shoot_failure(ctx, &verdict);
            return;
        }

        let Some(step) = self.steps.front().cloned() else {
            self.finish(ctx, "PASS");
            return;
        };

        match self.attempt(ctx, &step) {
            Outcome::Done => {
                self.steps.pop_front();
                self.number += 1;
                self.started = Instant::now();
            }
            Outcome::Waiting if self.started.elapsed() > self.patience => {
                self.fail(format!("FAIL step {} ({step:?}): timed out", self.number));
            }
            Outcome::Waiting => {}
            Outcome::Failed(reason) => {
                self.fail(format!("FAIL step {} ({step:?}): {reason}", self.number));
            }
        }
    }

    fn fail(&mut self, verdict: String) {
        self.failed = Some(verdict);
        self.shot = None;
        self.started = Instant::now();
    }

    /// The verdict waits for the shot, but never longer than a step may.
    fn shoot_failure(&mut self, ctx: &Context, verdict: &str) {
        match self.shoot(ctx, "failed") {
            Outcome::Waiting if self.started.elapsed() <= self.patience => {}
            Outcome::Failed(reason) => {
                tracing::error!("no shot of the failure: {reason}");
                self.finish(ctx, verdict);
            }
            Outcome::Done | Outcome::Waiting => self.finish(ctx, verdict),
        }
    }

    fn attempt(&mut self, ctx: &Context, step: &Step) -> Outcome {
        match *step {
            Step::Wait { role, label } => done_if(self.find(role, label).is_some()),
            Step::WaitContaining(text) => done_if(self.containing(text)),
            Step::WaitEither(one, other) => {
                done_if(self.find(None, one).is_some() || self.find(None, other).is_some())
            }
            Step::WaitGone(label) => done_if(self.find(None, label).is_none()),
            Step::Click { role, label } => self.click(role, label),
            Step::Text(text) => {
                self.events.push(Event::Text(text.to_owned()));
                Outcome::Done
            }
            Step::Key { modifiers, key } => {
                self.key(modifiers, key);
                Outcome::Done
            }
            Step::Disabled { role, label } => {
                done_if(self.find(role, label).is_some_and(|target| target.disabled))
            }
            Step::Shot(name) => self.shoot(ctx, name),
        }
    }

    fn find(&self, role: Option<Role>, label: &str) -> Option<Target> {
        let root = Found(self.tree.as_ref()?.root());
        let node = role.map_or_else(
            || root.query_all_by_label(label).next(),
            |role| root.query_all_by_role_and_label(role, label).next(),
        )?;

        let (node_id, tree_id) = node.0.locate();
        Some(Target {
            node: node_id,
            tree: tree_id,
            disabled: node.0.is_disabled(),
        })
    }

    fn containing(&self, text: &str) -> bool {
        self.tree.as_ref().is_some_and(|tree| {
            Found(tree.root())
                .query_all_by_label_contains(text)
                .next()
                .is_some()
        })
    }

    fn click(&mut self, role: Option<Role>, label: &str) -> Outcome {
        let Some(target) = self.find(role, label) else {
            return Outcome::Waiting;
        };

        // A field takes the focus, everything else the click a screen
        // reader sends, so no layer above it gets in the way.
        let action = if role == Some(Role::TextInput) {
            Action::Focus
        } else {
            Action::Click
        };

        self.events
            .push(Event::AccessKitActionRequest(ActionRequest {
                action,
                target_node: target.node,
                target_tree: target.tree,
                data: None,
            }));
        Outcome::Done
    }

    fn key(&mut self, modifiers: Modifiers, key: egui::Key) {
        for pressed in [true, false] {
            self.events.push(Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            });
        }
    }

    fn shoot(&mut self, ctx: &Context, name: &str) -> Outcome {
        let still = !settled(self.started.elapsed());
        if self.shot.is_none() && still {
            ctx.request_repaint();
            return Outcome::Waiting;
        }

        let Some(shot) = &self.shot else {
            let (sender, receiver) = crossbeam_channel::bounded(1);
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(
                move |image| drop(sender.send(image)),
            )));
            self.shot = Some(receiver);
            return Outcome::Waiting;
        };

        let Ok(image) = shot.try_recv() else {
            return Outcome::Waiting;
        };

        self.shot = None;
        match save(&self.dir.join(format!("{name}.png")), &image) {
            Ok(()) => Outcome::Done,
            Err(reason) => Outcome::Failed(reason),
        }
    }

    fn finish(&mut self, ctx: &Context, verdict: &str) {
        tracing::info!("self-test: {verdict}");
        if let Err(error) = std::fs::write(self.dir.join(RESULT_FILE), format!("{verdict}\n")) {
            tracing::error!("cannot write the self-test verdict: {error}");
        }

        self.finished = true;
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

impl Plugin for Runner {
    fn debug_name(&self) -> &'static str {
        "selftest"
    }

    fn input_hook(&mut self, ctx: &Context, input: &mut RawInput) {
        // The tree is only built while someone reads it.
        ctx.enable_accesskit();
        input.events.append(&mut self.events);
    }

    fn output_hook(&mut self, ctx: &Context, output: &mut FullOutput) {
        if let Some(update) = &output.platform_output.accesskit_update {
            match &mut self.tree {
                Some(tree) => tree.update(update.clone()),
                None => self.tree = Some(State::new(update.clone())),
            }
        }

        if !self.finished {
            self.advance(ctx);
            ctx.request_repaint();
        }
    }
}

const fn done_if(done: bool) -> Outcome {
    if done {
        Outcome::Done
    } else {
        Outcome::Waiting
    }
}

fn save(path: &Path, image: &ColorImage) -> Result<(), String> {
    let [width, height] = image.size;
    let width = u32::try_from(width).map_err(|error| error.to_string())?;
    let height = u32::try_from(height).map_err(|error| error.to_string())?;

    image::save_buffer(path, image.as_raw(), width, height, image::ColorType::Rgba8)
        .map_err(|error| format!("cannot save {}: {error}", path.display()))
}

/// Whether `elapsed` since the last step lets every slide and fade end.
pub(super) fn settled(elapsed: Duration) -> bool {
    elapsed > SETTLE
}
