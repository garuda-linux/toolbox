use std::time::Duration;

use super::{MAX_UNREAD, OutputGate};

#[tokio::test]
async fn output_waits_once_the_unread_bytes_reach_the_limit() {
    let gate = OutputGate::default();
    gate.enter(MAX_UNREAD).await;

    let waiting = tokio::time::timeout(Duration::from_millis(50), gate.enter(1)).await;
    assert!(
        waiting.is_err(),
        "the full queue holds the next output back"
    );

    gate.leave(1);
    tokio::time::timeout(Duration::from_secs(5), gate.enter(1))
        .await
        .unwrap();
}

#[tokio::test]
async fn a_chunk_above_the_limit_waits_for_an_empty_queue_only() {
    let gate = OutputGate::default();

    tokio::time::timeout(Duration::from_secs(5), gate.enter(MAX_UNREAD * 3))
        .await
        .unwrap();
}

#[test]
fn reading_frees_no_more_than_was_taken() {
    let gate = OutputGate::default();
    gate.leave(4096);

    assert_eq!(gate.0.available_permits(), MAX_UNREAD);
}
