use std::sync::Arc;
use std::time::{Duration, Instant};

use toolbox_system::share::Upload;

use super::spawn;
use crate::diagnostics::DryRunTransport;

#[test]
fn a_share_runs_once_at_a_time_and_ends_with_the_link() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let link = spawn(
        runtime.handle(),
        crate::waking::Wake::none(),
        Arc::new(DryRunTransport),
    );

    let mut upload = Upload::default();

    assert!(
        !link.share(&mut upload, String::new()),
        "an empty text went out"
    );
    assert!(link.share(&mut upload, "the output".to_owned()));
    assert_eq!(upload, Upload::Running);
    assert!(!link.share(&mut upload, "again".to_owned()));

    let start = Instant::now();
    while !link.take(&mut upload) {
        assert!(start.elapsed() < Duration::from_secs(10), "no link came");
        std::thread::sleep(Duration::from_millis(5));
    }

    let shared = upload.paste().map(|paste| paste.link.as_str());
    assert!(
        shared
            .is_some_and(|text| text.starts_with("https://bin.garudalinux.org/?dryrun0000000000#")),
        "{upload:?}"
    );
}
