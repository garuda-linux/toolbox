//! Which install step of the running plan failed on a file its mirror no
//! longer has, so the Console can offer a system update and the same
//! install again.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use toolbox_core::features::packages::mirror::mirror_missing;
use toolbox_plan::{PackageName, Plan, PlannedAction, RootAction, StepId, TargetList};

/// pacman names the missing file near its end, so an install step keeps
/// only this much of its newest output.
const KEPT_OUTPUT: usize = 64 * 1024;

#[derive(Debug, Default)]
pub struct MirrorWatch {
    sent: Option<BTreeMap<StepId, TargetList<PackageName>>>,
    installs: BTreeMap<StepId, Install>,
    offer: Option<TargetList<PackageName>>,
}

#[derive(Debug)]
struct Install {
    packages: TargetList<PackageName>,
    output: String,
}

impl MirrorWatch {
    pub fn sent(&mut self, plan: &Plan) {
        self.sent = Some(
            plan.steps()
                .iter()
                .filter_map(|step| match &step.action {
                    PlannedAction::Root(RootAction::InstallPackages { packages }) => {
                        Some((step.id, packages.clone()))
                    }
                    PlannedAction::Root(_) | PlannedAction::User(_) => None,
                })
                .collect(),
        );
    }

    pub fn started(&mut self) {
        self.installs = self
            .sent
            .take()
            .unwrap_or_default()
            .into_iter()
            .map(|(id, packages)| {
                let install = Install {
                    packages,
                    output: String::new(),
                };

                (id, install)
            })
            .collect();
        self.offer = None;
    }

    pub fn refused(&mut self) {
        self.sent = None;
    }

    pub fn output(&mut self, step: StepId, data: &str) {
        let Some(install) = self.installs.get_mut(&step) else {
            return;
        };

        install.output.push_str(data);
        if install.output.len() > KEPT_OUTPUT * 2 {
            let cut = install.output.len() - KEPT_OUTPUT;
            let cut = (cut..install.output.len())
                .find(|&at| install.output.is_char_boundary(at))
                .unwrap_or(cut);

            install.output.drain(..cut);
        }
    }

    pub fn finished(&mut self, step: StepId, succeeded: bool) {
        let Some(install) = self.installs.remove(&step) else {
            return;
        };

        if !succeeded && mirror_missing(&install.output) {
            self.offer = Some(install.packages);
        }
    }

    pub fn clear(&mut self) {
        self.installs.clear();
        self.offer = None;
    }

    pub const fn offer(&self) -> Option<&TargetList<PackageName>> {
        self.offer.as_ref()
    }
}
