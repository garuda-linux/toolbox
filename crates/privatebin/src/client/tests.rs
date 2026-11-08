use parking_lot::Mutex;
use serde_json::Value;

use super::{Paste, Transport, UploadError, upload, upload_with};
use crate::RequestError;
use crate::paste::Secrets;

const ID: &str = "0123456789abcdef";
const TOKEN: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

/// Answers every request with `answer` and keeps what it was sent.
struct Fake {
    answer: Result<String, RequestError>,
    requests: Mutex<Vec<(String, String)>>,
}

impl Fake {
    fn answering(answer: &str) -> Self {
        Self {
            answer: Ok(answer.to_owned()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<(String, String)> {
        self.requests.lock().clone()
    }
}

impl Transport for Fake {
    fn post(&self, url: &str, body: &str) -> Result<String, RequestError> {
        self.requests.lock().push((url.to_owned(), body.to_owned()));
        self.answer.clone()
    }
}

fn secrets() -> Secrets {
    Secrets {
        key: [7; 32],
        iv: [8; 16],
        salt: [9; 8],
    }
}

fn created() -> String {
    format!(r#"{{"status":0,"id":"{ID}","url":"/?{ID}","deletetoken":"{TOKEN}"}}"#)
}

#[test]
fn a_created_paste_gives_the_link_with_the_key_and_the_delete_link() {
    let fake = Fake::answering(&created());
    let paste = upload_with(&fake, "report", &secrets()).unwrap();

    assert_eq!(
        paste,
        Paste {
            link: format!("https://bin.garudalinux.org/?{ID}#{}", secrets().fragment()),
            delete_link: format!("https://bin.garudalinux.org/?pasteid={ID}&deletetoken={TOKEN}"),
        }
    );
}

#[test]
fn a_created_paste_does_not_print_its_key() {
    let fake = Fake::answering(&created());
    let paste = upload_with(&fake, "report", &secrets()).unwrap();
    let printed = format!("{paste:?}");

    assert!(!printed.contains(&secrets().fragment()), "{printed}");
    assert!(!printed.contains(TOKEN), "{printed}");
    assert!(printed.contains(ID), "{printed}");
}

#[test]
fn one_request_goes_to_the_fixed_server_without_the_key_or_the_text() {
    let fake = Fake::answering(&created());

    upload_with(&fake, "a very secret report", &secrets()).unwrap();

    let requests = fake.requests();
    let (url, body) = requests.first().unwrap();

    assert_eq!(requests.len(), 1);
    assert_eq!(url, "https://bin.garudalinux.org/");
    assert!(!body.contains("secret report"));
    assert!(!body.contains(&secrets().fragment()));

    let body: Value = serde_json::from_str(body).unwrap();

    assert_eq!(body["v"], 2_u8);
    assert_eq!(body["meta"]["expire"], "1year");
    assert!(body["ct"].as_str().is_some_and(|ct| !ct.is_empty()));
}

#[test]
fn fresh_secrets_make_every_link_differ() {
    let fake = Fake::answering(&created());
    let first = upload(&fake, "report").unwrap();
    let second = upload(&fake, "report").unwrap();

    assert_ne!(first.link, second.link);
    assert_ne!(fake.requests()[0].1, fake.requests()[1].1);
}

#[test]
fn a_refusal_carries_the_servers_message_as_plain_text() {
    let fake = Fake::answering(
        r#"{"status":1,"message":"Paste is limited to 10 MiB\n of <b>encrypted</b> data."}"#,
    );

    assert_eq!(
        upload_with(&fake, "report", &secrets()),
        Err(UploadError::Refused(
            "Paste is limited to 10 MiB of <b>encrypted</b> data.".to_owned()
        ))
    );

    let long = format!(r#"{{"status":1,"message":"{}"}}"#, "x".repeat(5000));

    match upload_with(&Fake::answering(&long), "report", &secrets()) {
        Err(UploadError::Refused(message)) => assert_eq!(message.len(), 200),
        other => panic!("{other:?}"),
    }
}

/// A proxy in front of PrivateBin answers a too large body with 413.
#[test]
fn an_error_status_is_a_refusal() {
    let rejecting = Fake {
        answer: Err(RequestError::Status(413)),
        requests: Mutex::new(Vec::new()),
    };

    assert_eq!(
        upload_with(&rejecting, "report", &secrets()),
        Err(UploadError::Refused("status 413".to_owned()))
    );
}

#[test]
fn an_id_that_would_change_the_link_is_refused() {
    for id in [
        "",
        "@evil.example/",
        "abc&deletetoken=1",
        "abc#frag",
        "../x",
        "a b",
        &"a".repeat(65),
    ] {
        let answer = serde_json::json!({ "status": 0_u8, "id": id, "deletetoken": TOKEN });
        let fake = Fake::answering(&answer.to_string());

        assert_eq!(
            upload_with(&fake, "report", &secrets()),
            Err(UploadError::BadAnswer),
            "{id}"
        );
    }

    let answer = serde_json::json!({ "status": 0_u8, "id": ID, "deletetoken": "x y" });

    assert_eq!(
        upload_with(&Fake::answering(&answer.to_string()), "report", &secrets()),
        Err(UploadError::BadAnswer)
    );
}

#[test]
fn the_servers_url_field_is_ignored() {
    let answer = serde_json::json!({
        "status": 0_u8,
        "id": ID,
        "url": "//evil.example/?x",
        "deletetoken": TOKEN,
    });

    let paste = upload_with(&Fake::answering(&answer.to_string()), "report", &secrets()).unwrap();

    assert!(
        paste
            .link
            .starts_with("https://bin.garudalinux.org/?0123456789abcdef#")
    );
    assert!(!paste.link.contains("evil"));
    assert!(!paste.delete_link.contains("evil"));
}

#[test]
fn no_answer_and_a_strange_answer_are_told_apart() {
    let offline = Fake {
        answer: Err(RequestError::NoAnswer("no route to host".to_owned())),
        requests: Mutex::new(Vec::new()),
    };

    assert_eq!(
        upload_with(&offline, "report", &secrets()),
        Err(UploadError::Transport(RequestError::NoAnswer(
            "no route to host".to_owned()
        )))
    );

    for answer in ["", "<html>502 Bad Gateway</html>", "[]", r#"{"id":"abc"}"#] {
        assert_eq!(
            upload_with(&Fake::answering(answer), "report", &secrets()),
            Err(UploadError::BadAnswer),
            "{answer}"
        );
    }
}

#[test]
fn the_delete_link_names_the_paste_and_its_token_without_the_key() {
    let paste = upload_with(&Fake::answering(&created()), "report", &secrets()).unwrap();

    assert!(!paste.delete_link.contains('#'));
    assert!(!paste.delete_link.contains(&secrets().fragment()));

    for token in ["", "a&pasteid=other", "a#b", "a/b", "a b", &"a".repeat(129)] {
        let answer = serde_json::json!({ "status": 0_u8, "id": ID, "deletetoken": token });

        assert_eq!(
            upload_with(&Fake::answering(&answer.to_string()), "report", &secrets()),
            Err(UploadError::BadAnswer),
            "{token}"
        );
    }
}

#[test]
fn a_refusal_strips_escapes_and_bidi_overrides() {
    let fake = Fake::answering(
        "{\"status\":1,\"message\":\"a\\u001b[31m b\\u202e c\\u2066 d\\u200e e\\u200f f\"}",
    );

    assert_eq!(
        upload_with(&fake, "report", &secrets()),
        Err(UploadError::Refused("a[31m b c d e f".to_owned()))
    );
}
