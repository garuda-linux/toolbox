use toolbox_plan::protocol::{Event, Mode, encode};

use super::{FakeLauncher, finished, root_plan, run};

/// A pipe names no caller, so the helper's stdin must be a socket.
#[tokio::test]
async fn the_helper_gets_a_socket_on_stdin() {
    let done = encode(&Event::Done).unwrap();
    let launcher = FakeLauncher {
        script: format!("test -S /dev/stdin && printf '%s' '{done}'"),
        mode: Mode::Run,
    };

    let reports = run(root_plan(), launcher).await;

    assert_eq!(reports.last(), Some(&finished(true)));
}
