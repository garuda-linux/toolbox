//! Secrets the helper hands to a program on its standard input: a new
//! account password or the passphrase of an encrypted partition. None of
//! these types can be printed, logged or serialized, so a secret cannot end
//! up in a request line, an event, the Console or a log by accident. The
//! one way out is [`SecretInput::expose`].

#[cfg(test)]
mod tests;

use std::fmt;

use crate::{StepId, UserName};

/// PAM's `PAM_MAX_RESP_SIZE`: longer answers are cut off by the modules.
pub const MAX_PASSWORD_BYTES: usize = 512;

const SECRET_PREFIX: &str = "secret ";

const REDACTED: &str = "<redacted>";

/// Bytes that are overwritten when dropped and never shown.
#[derive(Clone, PartialEq, Eq)]
struct Hidden(Vec<u8>);

impl Drop for Hidden {
    fn drop(&mut self) {
        self.0.fill(0);

        // Without a reader the optimiser may drop the fill of a buffer
        // that is freed right after.
        std::hint::black_box(&self.0);
    }
}

impl fmt::Debug for Hidden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

/// Never holds the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidPassword {
    #[error("the password is empty")]
    Empty,
    #[error("the password is longer than {max} bytes")]
    TooLong { max: usize },
    /// chpasswd reads one `user:password` per line, so a line break would
    /// start a second entry for another account.
    #[error("the password holds a line break or another control character")]
    ControlCharacter,
}

impl InvalidPassword {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::TooLong { max: _ } => "too-long",
            Self::ControlCharacter => "control-character",
        }
    }
}

/// A secret the user typed: a new account password or the passphrase of an
/// encrypted partition.
///
/// Not empty, at most [`MAX_PASSWORD_BYTES`] and without a control
/// character. It implements neither `Display` nor `Serialize`, and its
/// `Debug` prints a placeholder.
#[derive(Clone, PartialEq, Eq)]
pub struct Password(Hidden);

impl Password {
    /// Takes the text over without copying it.
    ///
    /// # Errors
    ///
    /// [`InvalidPassword`] for an empty or overlong text, or one with a
    /// control character.
    pub fn new(text: String) -> Result<Self, InvalidPassword> {
        let checked = check(&text);

        // Wrapped before the answer, so a refused text is overwritten too.
        let hidden = Hidden(text.into_bytes());

        checked.map(|()| Self(hidden))
    }

    pub fn chpasswd_input(&self, user: &UserName) -> SecretInput {
        self.line(&format!("{user}:"))
    }

    /// What `cryptsetup --key-file -` reads: the passphrase and nothing
    /// else. A line break at the end would become part of the key.
    pub fn key_input(&self) -> SecretInput {
        self.input(self.0.0.clone())
    }

    fn input(&self, bytes: Vec<u8>) -> SecretInput {
        SecretInput {
            bytes: Hidden(bytes),
            secret: self.0.clone(),
        }
    }

    /// The line that carries this password to the helper, for the step
    /// that needs it. It follows the request line, outside its JSON.
    pub fn wire_line(&self, step: StepId) -> SecretInput {
        self.line(&format!("{SECRET_PREFIX}{step} "))
    }

    fn line(&self, prefix: &str) -> SecretInput {
        // Sized up front: growing would leave a copy of the password in
        // freed memory.
        let mut line = Vec::with_capacity(prefix.len() + self.0.0.len() + 1);

        line.extend_from_slice(prefix.as_bytes());
        line.extend_from_slice(&self.0.0);
        line.push(b'\n');
        self.input(line)
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password({REDACTED})")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretInput {
    bytes: Hidden,
    /// The part of it that must never show. Kept apart because a
    /// passphrase may hold any character the input uses as a separator.
    secret: Hidden,
}

impl SecretInput {
    /// Only for writing them to the pipe of the program (or the helper) that
    /// needs them.
    pub fn expose(&self) -> &[u8] {
        &self.bytes.0
    }

    /// `text` with every occurrence of the secret replaced, so a program
    /// that echoes its input cannot leak it into the Console.
    pub fn scrub(&self, text: &str) -> String {
        match str::from_utf8(&self.secret.0) {
            Ok(secret) if !secret.is_empty() => text.replace(secret, REDACTED),
            Ok(_) | Err(_) => text.to_owned(),
        }
    }
}

impl fmt::Debug for SecretInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretInput({REDACTED})")
    }
}

/// Never holds the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidSecretLine {
    #[error("the line is no secret line")]
    Malformed,
    #[error("the secret is not valid: {0}")]
    Password(#[from] InvalidPassword),
}

/// Reads a line made by [`Password::wire_line`] and overwrites `line`
/// afterwards, whatever it held.
///
/// # Errors
///
/// [`InvalidSecretLine`] when the line has another shape or its password
/// breaks a rule.
pub fn parse_secret_line(line: Vec<u8>) -> Result<(StepId, Password), InvalidSecretLine> {
    let line = Hidden(line);
    let text = str::from_utf8(&line.0).map_err(|_unreadable| InvalidSecretLine::Malformed)?;
    let text = text.strip_suffix('\n').unwrap_or(text);
    let rest = text
        .strip_prefix(SECRET_PREFIX)
        .ok_or(InvalidSecretLine::Malformed)?;

    let (step, password) = rest.split_once(' ').ok_or(InvalidSecretLine::Malformed)?;
    let step: u32 = step
        .parse()
        .map_err(|_unreadable| InvalidSecretLine::Malformed)?;

    Ok((StepId::new(step), Password::new(password.to_owned())?))
}

fn check(text: &str) -> Result<(), InvalidPassword> {
    if text.is_empty() {
        return Err(InvalidPassword::Empty);
    }

    if text.len() > MAX_PASSWORD_BYTES {
        return Err(InvalidPassword::TooLong {
            max: MAX_PASSWORD_BYTES,
        });
    }

    if text.chars().any(char::is_control) {
        return Err(InvalidPassword::ControlCharacter);
    }

    Ok(())
}
