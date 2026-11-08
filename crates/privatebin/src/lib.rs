//! Uploads a text to Garuda's PrivateBin as an encrypted paste. The server
//! only ever sees ciphertext: the key is made here and lives in the
//! fragment of the returned link, which browsers never send.

mod client;
mod https;
pub mod paste;

pub use client::{Paste, RequestError, SERVER, Transport, UploadError, upload};
pub use https::Https;
