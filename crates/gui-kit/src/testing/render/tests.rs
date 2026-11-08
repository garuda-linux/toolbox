use super::super::harness;

#[test]
fn a_harness_leaves_the_software_renderer_one_thread() {
    let preset = std::env::var_os("LP_NUM_THREADS");
    drop(harness::<()>(100.0, 100.0).build_ui(|_| {}));

    let now = std::env::var_os("LP_NUM_THREADS");
    match preset {
        Some(preset) => assert_eq!(now, Some(preset)),
        None => assert_eq!(now.as_deref(), Some("1".as_ref())),
    }
}
