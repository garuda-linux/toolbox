//! The PrivateBin v2 paste format, without any I/O.
//!
//! The text becomes `{"paste": …}`, is deflated (raw, no zlib header) and
//! encrypted with AES-256-GCM. The AES key comes from PBKDF2-HMAC-SHA256
//! over the paste key, and the JSON text of the paste's settings is the
//! authenticated data, so the server cannot change them unnoticed.

#[cfg(test)]
mod tests;

use aes_gcm::aead::{Aead as _, Payload};
use aes_gcm::aes::Aes256;
use aes_gcm::aes::cipher::consts::U16;
use aes_gcm::{AesGcm, Key, KeyInit as _, Nonce};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde::Serialize;
use sha2::Sha256;

pub(crate) const KEY_BYTES: usize = 32;
/// PrivateBin uses a 16 byte nonce, not the 12 bytes common for GCM.
pub(crate) const IV_BYTES: usize = 16;
pub(crate) const SALT_BYTES: usize = 8;
pub(crate) const ITERATIONS: u32 = 100_000;
const KEY_BITS: u32 = 256;
const TAG_BITS: u32 = 128;
/// zlib's default.
const DEFLATE_LEVEL: u8 = 6;
const EXPIRE: &str = "1month";

type Cipher = AesGcm<Aes256, U16>;

/// The random parts of one paste. The key never reaches the server: it
/// travels in the link's fragment only.
pub(crate) struct Secrets {
    pub(crate) key: [u8; KEY_BYTES],
    pub(crate) iv: [u8; IV_BYTES],
    pub(crate) salt: [u8; SALT_BYTES],
}

/// `[iv, salt, iterations, key bits, tag bits, "aes", "gcm", "zlib"]`
#[derive(Serialize)]
struct Spec(
    String,
    String,
    u32,
    u32,
    u32,
    &'static str,
    &'static str,
    &'static str,
);

/// `[spec, format, open discussion, burn after reading]`. Its compact JSON
/// text is what AES-GCM authenticates.
#[derive(Serialize)]
pub(crate) struct Adata(Spec, &'static str, u8, u8);

#[derive(Serialize)]
struct Meta {
    expire: &'static str,
}

#[derive(Serialize)]
pub(crate) struct PasteRequest {
    #[serde(rename = "v")]
    version: u8,
    ct: String,
    adata: Adata,
    meta: Meta,
}

#[derive(Serialize)]
struct Content<'text> {
    paste: &'text str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SealError {
    #[error("the system gave no random numbers")]
    NoRandomness,
    #[error("the text cannot be encrypted")]
    Encryption,
}

impl Secrets {
    pub(crate) fn random() -> Result<Self, SealError> {
        let mut secrets = Self {
            key: [0; KEY_BYTES],
            iv: [0; IV_BYTES],
            salt: [0; SALT_BYTES],
        };

        for part in [
            secrets.key.as_mut_slice(),
            secrets.iv.as_mut_slice(),
            secrets.salt.as_mut_slice(),
        ] {
            getrandom::fill(part).map_err(|_unavailable| SealError::NoRandomness)?;
        }

        Ok(secrets)
    }

    /// The key as the link's fragment (base58, Bitcoin's alphabet).
    pub(crate) fn fragment(&self) -> String {
        bs58::encode(self.key).into_string()
    }
}

impl PasteRequest {
    pub(crate) fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Fails with [`SealError::Encryption`] for a text beyond AES-GCM's size
/// limit.
pub(crate) fn seal(text: &str, secrets: &Secrets) -> Result<PasteRequest, SealError> {
    encrypt(&compress(text)?, secrets)
}

/// `{"paste": text}` as raw deflate.
fn compress(text: &str) -> Result<Vec<u8>, SealError> {
    let content = serde_json::to_vec(&Content { paste: text })
        .map_err(|_unserializable| SealError::Encryption)?;

    Ok(miniz_oxide::deflate::compress_to_vec(
        &content,
        DEFLATE_LEVEL,
    ))
}

/// Encrypts the compressed content. Apart from the secrets the result
/// depends on nothing, so a known answer can be compared byte for byte.
fn encrypt(compressed: &[u8], secrets: &Secrets) -> Result<PasteRequest, SealError> {
    let adata = Adata(
        Spec(
            STANDARD.encode(secrets.iv),
            STANDARD.encode(secrets.salt),
            ITERATIONS,
            KEY_BITS,
            TAG_BITS,
            "aes",
            "gcm",
            "zlib",
        ),
        "plaintext",
        0,
        0,
    );

    let authenticated =
        serde_json::to_vec(&adata).map_err(|_unserializable| SealError::Encryption)?;

    let derived =
        pbkdf2::pbkdf2_hmac_array::<Sha256, KEY_BYTES>(&secrets.key, &secrets.salt, ITERATIONS);

    let sealed = Cipher::new(&Key::<Cipher>::from(derived))
        .encrypt(
            &Nonce::<U16>::from(secrets.iv),
            Payload {
                msg: compressed,
                aad: &authenticated,
            },
        )
        .map_err(|_too_long| SealError::Encryption)?;

    Ok(PasteRequest {
        version: 2,
        ct: STANDARD.encode(sealed),
        adata,
        meta: Meta { expire: EXPIRE },
    })
}
