use std::sync::Arc;

use super::{RowCache, Same};

#[test]
fn equal_inputs_keep_the_rows() {
    let mut cache: RowCache<u8, Vec<u8>> = RowCache::default();
    let first = cache.get(&["fire"], 1, || vec![1]);
    let second = cache.get(&["fire"], 1, || vec![2]);

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(cache.builds(), 1);
}

#[test]
fn a_new_text_or_input_builds_again() {
    let mut cache: RowCache<u8, Vec<u8>> = RowCache::default();
    let _: Arc<Vec<u8>> = cache.get(&["fire"], 1, Vec::new);
    let _: Arc<Vec<u8>> = cache.get(&["firef"], 1, Vec::new);
    let _: Arc<Vec<u8>> = cache.get(&["firef"], 2, Vec::new);

    assert_eq!(cache.builds(), 3);
}

#[test]
fn a_value_is_the_same_only_as_itself() {
    let one = Arc::new(1_u8);
    let equal = Arc::new(1_u8);

    assert_eq!(Same::of(&one), Same::of(&Arc::clone(&one)));
    assert_ne!(Same::of(&one), Same::of(&equal));
}
