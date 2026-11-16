use std::io;
use std::sync::Arc;

use toolbox_https::ClientError;
use toolbox_privatebin::paste::SealError;
use toolbox_privatebin::{RequestError, UploadError};
use toolbox_state::diagnostics::UploadFailure;

use super::UploadProblem;

fn problem(kind: UploadFailure, detail: &str) -> UploadProblem {
    UploadProblem {
        kind,
        detail: detail.to_owned(),
    }
}

#[test]
fn each_upload_error_has_its_code_and_the_words_behind_it() {
    let cases = [
        (
            UploadError::Transport(RequestError::NoAnswer("dns failed".to_owned())),
            problem(UploadFailure::Network, "dns failed"),
        ),
        (
            UploadError::Transport(RequestError::Unreadable(Arc::new(ClientError::Io(
                io::Error::other("reset"),
            )))),
            problem(
                UploadFailure::Network,
                "the answer could not be read: io: reset",
            ),
        ),
        (
            UploadError::Refused(toolbox_text::plain("Paste is limited to 10 MiB")),
            problem(UploadFailure::Refused, "Paste is limited to 10 MiB"),
        ),
        (
            UploadError::BadAnswer,
            problem(UploadFailure::BadAnswer, ""),
        ),
        (
            UploadError::Seal(SealError::NoRandomness),
            problem(UploadFailure::Seal, "the system gave no random numbers"),
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(UploadProblem::from(error), expected);
    }
}

#[test]
fn an_upload_that_stopped_has_no_words() {
    assert_eq!(
        UploadProblem::stopped(),
        problem(UploadFailure::Stopped, "")
    );
}
