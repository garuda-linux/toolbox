use super::{InvalidPassword, InvalidSecretLine, MAX_PASSWORD_BYTES, Password, parse_secret_line};
use crate::{StepId, UserName};

const SECRET: &str = "correct horse: battery staple";

fn password() -> Password {
    Password::new(SECRET.to_owned()).unwrap()
}

#[test]
fn a_password_never_prints_itself() {
    let password = password();
    let user = UserName::new("nico").unwrap();

    assert_eq!(format!("{password:?}"), "Password(<redacted>)");
    assert_eq!(
        format!("{:?}", Some(&password)),
        "Some(Password(<redacted>))"
    );
    assert_eq!(
        format!("{:?}", password.chpasswd_input(&user)),
        "SecretInput(<redacted>)"
    );
    assert_eq!(
        format!("{:#?}", password.wire_line(StepId::new(2))),
        "SecretInput(<redacted>)"
    );
}

#[test]
fn empty_overlong_and_multi_line_passwords_are_refused() {
    assert_eq!(Password::new(String::new()), Err(InvalidPassword::Empty));
    assert_eq!(
        Password::new("x".repeat(MAX_PASSWORD_BYTES + 1)),
        Err(InvalidPassword::TooLong {
            max: MAX_PASSWORD_BYTES
        })
    );
    Password::new("x".repeat(MAX_PASSWORD_BYTES)).unwrap();

    for text in ["a\nroot:b", "a\rb", "a\0b", "tab\there", "\u{85}"] {
        assert_eq!(
            Password::new(text.to_owned()),
            Err(InvalidPassword::ControlCharacter),
            "{}",
            text.escape_debug()
        );
    }
}

#[test]
fn an_invalid_password_error_does_not_quote_the_text() {
    let error = Password::new("hunter2\nroot:hunter2".to_owned()).unwrap_err();

    assert!(!error.to_string().contains("hunter2"));
    assert!(!format!("{error:?}").contains("hunter2"));
    assert_eq!(error.as_str(), "control-character");
}

#[test]
fn spaces_colons_and_other_scripts_are_fine() {
    for text in [" lead and trail ", "a:b:c", "pässwört", "密码", "'\"$(id)`"] {
        Password::new(text.to_owned()).unwrap();
    }
}

#[test]
fn chpasswd_reads_one_line_of_user_and_password() {
    let user = UserName::new("nico").unwrap();

    assert_eq!(
        password().chpasswd_input(&user).expose(),
        format!("nico:{SECRET}\n").as_bytes()
    );
}

#[test]
fn the_wire_line_round_trips() {
    let line = password().wire_line(StepId::new(7));

    assert_eq!(line.expose(), format!("secret 7 {SECRET}\n").as_bytes());
    assert_eq!(
        parse_secret_line(line.expose().to_vec()),
        Ok((StepId::new(7), password()))
    );
}

#[test]
fn other_lines_are_no_secret_lines() {
    for line in [
        "",
        "secret",
        "secret 1",
        "secret x hunter2",
        "{\"type\":\"cancel\"}",
        "secret -1 hunter2",
    ] {
        assert_eq!(
            parse_secret_line(line.as_bytes().to_vec()),
            Err(InvalidSecretLine::Malformed),
            "{line}"
        );
    }

    assert_eq!(
        parse_secret_line(b"secret 1 \n".to_vec()),
        Err(InvalidSecretLine::Password(InvalidPassword::Empty))
    );
    assert_eq!(
        parse_secret_line(vec![b's', 0xff, b'\n']),
        Err(InvalidSecretLine::Malformed)
    );
}

#[test]
fn a_secret_line_error_does_not_quote_the_line() {
    let error = parse_secret_line(b"secret 1 hunter2\x07\n".to_vec()).unwrap_err();

    assert!(!error.to_string().contains("hunter2"));
}

#[test]
fn scrubbing_removes_the_secret_from_program_output() {
    let user = UserName::new("nico").unwrap();
    let input = password().chpasswd_input(&user);

    assert_eq!(
        input.scrub(&format!("chpasswd: bad line nico:{SECRET}\r\n")),
        "chpasswd: bad line nico:<redacted>\r\n"
    );
    assert_eq!(input.scrub("nothing to hide"), "nothing to hide");
}

#[test]
fn hidden_bytes_print_redacted() {
    assert_eq!(
        format!("{:?}", super::Hidden(b"secret".to_vec())),
        "<redacted>"
    );
}
