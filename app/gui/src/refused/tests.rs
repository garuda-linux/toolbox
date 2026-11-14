use toolbox_core::domains::Packages;
use toolbox_core::features::kernels;
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::{Refused, Shown as _, queue_or_refuse};

fn firefox() -> PackageName {
    "firefox".parse().unwrap()
}

#[test]
fn a_taken_click_is_an_edit_and_clears_the_refusal() {
    let mut slot = Some(Refused {
        refusal: kernels::Refusal::LastKernel,
        subject: firefox(),
    });

    let handled = queue_or_refuse(&mut slot, firefox(), &ChangeSet::default(), |changes| {
        changes.want::<Packages>(firefox(), Toggle::On, &Snapshot::default());
        Ok(())
    });

    assert!(slot.is_none());
    assert_eq!(handled.edit.map(|edit| edit.add.len()), Some(1));
}

#[test]
fn a_refused_click_keeps_the_queue_and_names_why() {
    let mut slot = None;

    let handled = queue_or_refuse(&mut slot, firefox(), &ChangeSet::default(), |changes| {
        changes.want::<Packages>(firefox(), Toggle::On, &Snapshot::default());
        Err(kernels::Refusal::Running)
    });

    assert!(handled.edit.is_none());
    assert_eq!(
        slot,
        Some(Refused {
            refusal: kernels::Refusal::Running,
            subject: firefox(),
        })
    );
}

#[test]
fn the_text_names_the_subject() {
    let ctx = egui::Context::default();
    let refused = Refused {
        refusal: kernels::Refusal::Running,
        subject: firefox(),
    };

    assert!(
        refused.text(&ctx).contains("firefox"),
        "{}",
        refused.text(&ctx)
    );
}
