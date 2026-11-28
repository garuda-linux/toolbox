use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use strum::IntoEnumIterator;
use toolbox_core::features::RefusalText;
use toolbox_system::Probe;

use super::{available, set_language, text, text_or};

fn ids_of(locale: &Path) -> BTreeSet<String> {
    fs::read_dir(locale)
        .unwrap()
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .flat_map(|file| {
            file.lines()
                .filter_map(|line| line.split_once(" =").map(|(id, _)| id.to_owned()))
                .filter(|id| {
                    id.chars()
                        .next()
                        .is_some_and(|first| first.is_ascii_lowercase())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn every_locale_has_the_same_message_ids() {
    let root = Path::new(toolbox_services::i18n::DIR);
    let english = ids_of(&root.join("en-US"));

    for locale in available() {
        assert_eq!(
            ids_of(&root.join(locale.to_string())),
            english,
            "{locale} differs from en-US"
        );
    }
}

#[test]
fn every_probe_has_a_name() {
    let english = ids_of(&Path::new(toolbox_services::i18n::DIR).join("en-US"));

    for probe in Probe::iter() {
        let id = format!("probe-{}", probe.as_str());
        assert!(english.contains(&id), "{id} is missing");
    }
}

#[test]
fn texts_come_in_the_language_set_on_the_context() {
    let ctx = egui::Context::default();
    assert_eq!(text(&ctx, "language-name"), "English (United States)");

    set_language(&ctx, &"de-DE".parse().unwrap());
    assert_eq!(text(&ctx, "language-name"), "Deutsch (Deutschland)");
}

#[test]
fn an_unknown_id_shows_itself() {
    let ctx = egui::Context::default();

    assert_eq!(text(&ctx, "no-such-message"), "no-such-message");
}

#[test]
fn catalog_texts_come_translated_or_as_the_catalog_says() {
    let ctx = egui::Context::default();
    set_language(&ctx, &"de-DE".parse().unwrap());

    assert_eq!(
        text_or(&ctx, "catalog-components-virtualization", "Virtualization"),
        "Virtualisierung"
    );
    assert_eq!(text_or(&ctx, "catalog-components-new", "New"), "New");
}

/// The Users page words a lock that could not be read by its id, in each
/// language, so a refusal without a text would show the bare id.
#[test]
fn every_lock_refusal_has_its_text_in_each_language() {
    use toolbox_system::pkexec::Refusal;

    use crate::pages::users::texts::refused_lock_id;

    let root = Path::new(toolbox_services::i18n::DIR);
    let refusals = [
        Refusal::Dismissed,
        Refusal::Denied,
        Refusal::NoAgent,
        Refusal::HelperMissing,
        Refusal::Broken,
    ];

    for locale in ["en-US", "de-DE"] {
        let ids = ids_of(&root.join(locale));
        for refusal in refusals {
            let id = refused_lock_id(refusal);
            assert!(ids.contains(id), "{id} is missing in {locale}");
        }
    }
}

#[test]
fn every_account_refusal_has_a_text() {
    let english = ids_of(&Path::new(toolbox_services::i18n::DIR).join("en-US"));

    for refusal in toolbox_core::features::users::Refusal::iter() {
        let id = format!("account-refusal-{}", refusal.code());
        assert!(english.contains(&id), "{id} is missing");
    }
}

/// A page shows each refusal by its `text_id`, so a variant without its
/// text would show the bare id.
#[test]
fn every_page_refusal_has_its_text() {
    use toolbox_core::features::{bootrepair, configs, kernels, locales, network, users};

    fn ids<R>() -> Vec<&'static str>
    where
        R: RefusalText + IntoEnumIterator,
    {
        R::iter().filter_map(RefusalText::text_id).collect()
    }

    let all = [
        ids::<users::Refusal>(),
        ids::<kernels::Refusal>(),
        ids::<locales::Refusal>(),
        ids::<network::Refusal>(),
        ids::<bootrepair::Refusal>(),
        ids::<configs::Refusal>(),
        ids::<configs::session::ProblemCode>(),
    ];

    assert_texts_in_each_language(&all.concat());
}

/// The Maintenance page names each task and its detail by `text_id`.
#[test]
fn every_maintenance_task_has_its_name_and_detail() {
    use toolbox_core::ChangeSet;
    use toolbox_core::features::maintenance::page::{Repair, operations, tools};

    let english = ids_of(&Path::new(toolbox_services::i18n::DIR).join("en-US"));
    let names = operations(&ChangeSet::default())
        .into_iter()
        .map(|row| row.operation.text_id())
        .chain(tools(|_| false).into_iter().map(|row| row.tool.text_id()))
        .chain(Repair::ALL.map(Repair::text_id));

    let missing: Vec<String> = names
        .flat_map(|id| [format!("{id}-detail"), id.to_owned()])
        .filter(|id| !english.contains(id))
        .collect();

    assert!(missing.is_empty(), "no text for {missing:?}");
}

#[test]
fn a_text_without_arguments_is_looked_up_once_per_language() {
    let ctx = egui::Context::default();
    let first = text(&ctx, "page-home");
    let again = text(&ctx, "page-home");

    set_language(&ctx, &"de-DE".parse().unwrap());
    let german = text(&ctx, "page-home");

    assert!(matches!(again, Cow::Borrowed(_)));
    assert!(std::ptr::eq(first.as_ptr(), again.as_ptr()));
    assert_ne!(first, german);
    assert!(std::ptr::eq(
        german.as_ptr(),
        text(&ctx, "page-home").as_ptr()
    ));
}

/// The toggle pages word a refusal by its `text_id`, so an account refusal
/// and a protection without its text would show the bare id.
#[test]
fn every_toggle_refusal_has_its_text_in_each_language() {
    use toolbox_core::features::toggles::Refusal;
    use toolbox_core::features::users;

    let mut ids: Vec<&str> = users::Refusal::iter()
        .filter_map(|refusal| Refusal::Account(refusal).text_id())
        .collect();

    ids.extend(["protected-refused", "protected-along"]);

    assert_texts_in_each_language(&ids);
}

/// The boot page words each rule a value breaks by its `text_id`.
#[test]
fn every_boot_rule_has_its_text_in_each_language() {
    use toolbox_core::features::ReasonCode;
    use toolbox_core::features::boot::edit::EditCode;

    let ids: Vec<&str> = ReasonCode::iter()
        .filter_map(|reason| EditCode::Rule(reason).text_id())
        .collect();

    assert_texts_in_each_language(&ids);
}

/// The Maintenance page asks each task and each repair its question by id.
#[test]
fn every_maintenance_question_has_its_text_in_each_language() {
    use toolbox_core::ChangeSet;
    use toolbox_core::features::maintenance::page::{Repair, operations};

    let ids: Vec<&str> = operations(&ChangeSet::default())
        .into_iter()
        .filter_map(|row| row.operation.question_id())
        .chain(Repair::ALL.into_iter().filter_map(Repair::question_id))
        .collect();

    assert_texts_in_each_language(&ids);
}

/// The probe failures name a phrase of the workspace by its `probe-subject`
/// id, so each one needs its text in both bundles.
#[test]
fn every_probe_subject_phrase_has_its_text_in_each_language() {
    assert_texts_in_each_language(&[
        "probe-subject-local-database",
        "probe-subject-disks",
        "probe-subject-running-kernel",
        "probe-subject-route-list",
        "probe-subject-build-order",
        "probe-subject-sample-build-order",
    ]);
}

/// The Diagnostics page words each reason of a failed collection by its id.
#[test]
fn every_failed_collection_reason_has_its_text_in_each_language() {
    assert_texts_in_each_language(&[
        "diagnostics-failed-no-program",
        "diagnostics-failed-exit-code",
        "diagnostics-failed-signal",
        "diagnostics-failed-start",
        "diagnostics-failed-timed-out",
        "diagnostics-failed-too-large",
        "diagnostics-failed-read",
        "diagnostics-failed-helper-missing",
        "diagnostics-failed-helper-start",
        "diagnostics-failed-helper-timed-out",
        "diagnostics-failed-helper-too-long",
        "diagnostics-failed-helper-pkexec",
        "diagnostics-failed-helper-ended",
        "diagnostics-failed-ending-code",
        "diagnostics-failed-ending-signal",
        "diagnostics-failed-helper-unreadable",
        "diagnostics-failed-helper-stopped",
        "diagnostics-failed-answer",
        "diagnostics-failed-answer-too-long",
        "diagnostics-failed-not-installed",
        "diagnostics-failed-dismissed",
        "diagnostics-failed-denied",
        "diagnostics-failed-no-agent",
        "diagnostics-failed-helper-not-installed",
    ]);
}

/// Apply's hold names one of these texts, the guards' ones per state of
/// their read and the rest from the app.
#[test]
fn every_apply_hold_has_its_text_in_each_language() {
    use toolbox_core::features::{Guard, ReadState};

    let states = [ReadState::Failed, ReadState::Waiting];
    let mut ids = vec![
        "changes-wait-running",
        "changes-wait-reading",
        "changes-wait-orders",
    ];

    for guard in Guard::iter() {
        for locks in states {
            for aur in states {
                ids.push(guard.hold_text(locks, aur));
            }
        }
    }

    ids.sort_unstable();
    ids.dedup();

    assert_texts_in_each_language(&ids);
}

fn assert_texts_in_each_language(ids: &[&str]) {
    let root = Path::new(toolbox_services::i18n::DIR);

    for locale in ["en-US", "de-DE"] {
        let shown = ids_of(&root.join(locale));
        let missing: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|id| !shown.contains(*id))
            .collect();

        assert!(missing.is_empty(), "no {locale} text for {missing:?}");
    }
}
