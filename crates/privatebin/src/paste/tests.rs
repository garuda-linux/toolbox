//! The known answer below was made outside this crate, in Node with
//! `WebCrypto` PBKDF2 and AES-GCM and zlib's raw deflate, from the fixed
//! key, nonce and salt of [`fixed`].

use aes_gcm::aead::{Aead as _, Payload};
use aes_gcm::aes::Aes256;
use aes_gcm::aes::cipher::consts::U16;
use aes_gcm::{AesGcm, KeyInit as _};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use sha2::Sha256;

use super::{ITERATIONS, PasteRequest, Secrets, encrypt, seal};

const TEXT: &str = "Garuda Toolbox known vector\nline two: äöü €\n";
const ADATA: &str = r#"[["oKGio6SlpqeoqaqrrK2urw==","EBESExQVFhc=",100000,256,128,"aes","gcm","zlib"],"plaintext",0,0]"#;
const NODE_COMPRESSED: &str =
    "q1YqSCwuSVWyUnJPLCpNSVQIyc/PScqvUMjOyy/PUyhLTS7JL4rJy8nMS1UoKc+3Uji85PC2w3sUHjWticlTqgUA";
const NODE_CT: &str = "g3v+2ccJy1tox4s/8f84w9uNhVUpizLHPmsuu49l4engfLWDaUJCK+kMk3ruo99JqA996Pgic+ChdHGZVZU7v3Ocd7RMMefA9Bsxm66PciiFCQ==";
const NODE_FRAGMENT: &str = "1thX6LZfHDZZKUs92febYZhYRcXddmzfzF2NvTkPNE";

fn fixed() -> Secrets {
    Secrets {
        key: std::array::from_fn(|index| u8::try_from(index).unwrap()),
        iv: std::array::from_fn(|index| 0xa0 + u8::try_from(index).unwrap()),
        salt: std::array::from_fn(|index| 0x10 + u8::try_from(index).unwrap()),
    }
}

/// A reader written apart from `encrypt`: it takes every parameter from
/// the request's JSON, as PrivateBin's own client does, and none from this
/// crate's constants.
fn open(request: &Value, key: &[u8]) -> String {
    let adata = &request["adata"];
    let spec = &adata[0];
    let iv = STANDARD.decode(spec[0].as_str().unwrap()).unwrap();
    let salt = STANDARD.decode(spec[1].as_str().unwrap()).unwrap();
    let iterations = u32::try_from(spec[2].as_u64().unwrap()).unwrap();

    assert_eq!(spec[3], 256_u32);
    assert_eq!(spec[4], 128_u32);
    assert_eq!(spec[5], "aes");
    assert_eq!(spec[6], "gcm");
    assert_eq!(spec[7], "zlib");

    let mut derived = [0_u8; 32];

    pbkdf2::pbkdf2_hmac::<Sha256>(key, &salt, iterations, &mut derived);

    let cipher = AesGcm::<Aes256, U16>::new_from_slice(&derived).unwrap();
    let compressed = cipher
        .decrypt(
            iv.as_slice().try_into().unwrap(),
            Payload {
                msg: &STANDARD.decode(request["ct"].as_str().unwrap()).unwrap(),
                aad: serde_json::to_string(adata).unwrap().as_bytes(),
            },
        )
        .unwrap();

    let content = miniz_oxide::inflate::decompress_to_vec(&compressed).unwrap();
    let content: Value = serde_json::from_slice(&content).unwrap();

    content["paste"].as_str().unwrap().to_owned()
}

fn as_value(request: &PasteRequest) -> Value {
    serde_json::from_str(&request.to_json().unwrap()).unwrap()
}

#[test]
fn the_same_compressed_bytes_encrypt_to_the_known_ciphertext() {
    let compressed = STANDARD.decode(NODE_COMPRESSED).unwrap();
    let request = encrypt(&compressed, &fixed()).unwrap();

    assert_eq!(request.ct, NODE_CT);
    assert_eq!(serde_json::to_string(&request.adata).unwrap(), ADATA);
}

#[test]
fn the_independent_reader_opens_the_known_paste() {
    let request = serde_json::json!({
        "v": 2_u8,
        "ct": NODE_CT,
        "adata": serde_json::from_str::<Value>(ADATA).unwrap(),
    });

    assert_eq!(open(&request, &fixed().key), TEXT);
}

#[test]
fn a_sealed_text_opens_again() {
    for text in [
        TEXT,
        "",
        "\u{1b}[31mred\u{1b}[0m \"quoted\" \\ \t tab",
        &"journal line\n".repeat(20_000),
    ] {
        let request = seal(text, &fixed()).unwrap();

        assert_eq!(open(&as_value(&request), &fixed().key), text);
    }
}

#[test]
fn the_request_has_privatebins_shape_and_options() {
    let request = as_value(&seal(TEXT, &fixed()).unwrap());

    assert_eq!(request["v"], 2_u8);
    assert_eq!(request["meta"], serde_json::json!({ "expire": "1year" }));
    assert_eq!(
        request["adata"],
        serde_json::from_str::<Value>(ADATA).unwrap()
    );
    assert_eq!(
        request.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["v", "ct", "adata", "meta"]
    );
}

#[test]
fn a_long_text_is_compressed() {
    let text = "the same line again\n".repeat(10_000);
    let request = seal(&text, &fixed()).unwrap();

    assert!(request.ct.len() < text.len() / 50, "{}", request.ct.len());
}

#[test]
fn a_changed_setting_or_key_does_not_open() {
    let request = as_value(&seal(TEXT, &fixed()).unwrap());
    let adata = &request["adata"];
    let derived = pbkdf2::pbkdf2_hmac_array::<Sha256, 32>(&fixed().key, &fixed().salt, ITERATIONS);

    let wrong_key = pbkdf2::pbkdf2_hmac_array::<Sha256, 32>(&[9; 32], &fixed().salt, ITERATIONS);
    let sealed = STANDARD.decode(request["ct"].as_str().unwrap()).unwrap();
    let aad = serde_json::to_string(adata).unwrap();
    let burn = aad.replace("\"plaintext\",0,0", "\"plaintext\",0,1");

    let try_open = |key: &[u8; 32], authenticated: &str| {
        AesGcm::<Aes256, U16>::new_from_slice(key)
            .unwrap()
            .decrypt(
                (&fixed().iv).into(),
                Payload {
                    msg: &sealed,
                    aad: authenticated.as_bytes(),
                },
            )
            .is_ok()
    };

    assert!(try_open(&derived, &aad));
    assert!(!try_open(&derived, &burn));
    assert!(!try_open(&wrong_key, &aad));
}

#[test]
fn the_fragment_is_the_key_in_base58() {
    assert_eq!(fixed().fragment(), NODE_FRAGMENT);
    assert_eq!(
        bs58::decode(fixed().fragment()).into_vec().unwrap(),
        fixed().key
    );
}

#[test]
fn random_secrets_differ() {
    let first = Secrets::random().unwrap();
    let second = Secrets::random().unwrap();

    assert_ne!(first.key, second.key);
    assert_ne!(first.iv, second.iv);
    assert_ne!(first.salt, second.salt);
    assert_ne!(first.key, [0; 32]);
}
