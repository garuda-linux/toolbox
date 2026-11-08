use std::sync::Arc;

use super::Loadable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Why {
    Denied,
}

fn finished<E>(result: Result<u8, E>) -> Loadable<u8, E> {
    let mut loadable = Loadable::default();
    loadable.finish(result);
    loadable
}

#[test]
fn nothing_is_loaded_at_first() {
    let loadable = Loadable::<u8>::default();

    assert_eq!(loadable, Loadable::NotLoaded);
    assert_eq!(loadable.get(), None);
    assert!(!loadable.is_loading());
}

#[test]
fn a_result_becomes_a_value_or_a_failure() {
    let loaded = finished::<String>(Ok(3));
    let failed = finished::<String>(Err("no bus".to_owned()));

    assert_eq!(loaded.get(), Some(&3));
    assert_eq!(failed.error().map(String::as_str), Some("no bus"));
    assert_eq!(failed.get(), None);
}

#[test]
fn the_error_keeps_its_own_type() {
    let mut loadable = finished::<Why>(Ok(3));

    loadable.fail(Why::Denied);

    assert_eq!(loadable.error(), Some(&Why::Denied));
    assert_eq!(loadable.get(), Some(&3));
    assert_eq!(loadable.clone(), loadable);
}

#[test]
fn loading_again_keeps_the_value_on_screen() {
    let mut loadable = finished::<String>(Ok(3));

    loadable.start_loading();

    assert!(loadable.is_loading());
    assert_eq!(loadable.get(), Some(&3));
}

#[test]
fn a_failed_load_keeps_the_last_value() {
    let mut loadable = finished::<String>(Ok(3));
    loadable.start_loading();

    loadable.finish(Err("timed out".to_owned()));

    assert_eq!(loadable.error().map(String::as_str), Some("timed out"));
    assert_eq!(loadable.get(), Some(&3));
    assert!(!loadable.is_loading());
}

#[test]
fn a_retry_after_a_failure_keeps_the_last_value() {
    let mut loadable = finished::<String>(Ok(3));
    loadable.fail("timed out".to_owned());

    loadable.start_loading();

    assert!(loadable.is_loading());
    assert_eq!(loadable.error(), None);
    assert_eq!(loadable.get(), Some(&3));
}

#[test]
fn a_finished_load_replaces_the_value() {
    let mut loadable = finished::<String>(Ok(3));
    loadable.start_loading();

    loadable.finish(Ok(4));

    assert_eq!(loadable, Loadable::Loaded(Arc::new(4)));
}

#[test]
fn a_shared_value_is_kept_without_a_copy() {
    let shared = Arc::new(vec![1_u8, 2]);
    let mut loadable = Loadable::<Vec<u8>>::NotLoaded;

    loadable.set_loaded(Arc::clone(&shared));

    assert!(
        loadable
            .latest()
            .is_some_and(|kept| Arc::ptr_eq(kept, &shared))
    );
}

#[test]
fn a_first_load_that_fails_has_nothing_to_show() {
    let mut loadable = Loadable::<u8>::NotLoaded;
    loadable.start_loading();
    assert_eq!(loadable, Loadable::Loading { previous: None });

    loadable.fail("denied".to_owned());

    assert_eq!(
        loadable,
        Loadable::Failed {
            error: "denied".to_owned(),
            previous: None
        }
    );
}
