//! JPEG XL pictures for egui's image loader. Arch ships its `AppStream`
//! icon cache in this format, and the `image` crate has no decoder of its
//! own, so `jxl-oxide` is registered as one of its hooks.

#[cfg(test)]
mod tests;

use std::sync::Once;

use jxl_oxide::integration::JxlDecoder;

/// The bare codestream and the ISO container start with these bytes.
const CODESTREAM: &[u8] = &[0xff, 0x0a];
const CONTAINER: &[u8] = &[
    0x00, 0x00, 0x00, 0x0c, 0x4a, 0x58, 0x4c, 0x20, 0x0d, 0x0a, 0x87, 0x0a,
];

/// Teaches the `image` crate JPEG XL by file ending and by its first
/// bytes. The hooks are global and a second registration would add the
/// signatures twice, so only the first call registers.
pub fn register() {
    static REGISTERED: Once = Once::new();

    REGISTERED.call_once(|| {
        image::hooks::register_decoding_hook(
            "jxl".into(),
            Box::new(|reader| Ok(Box::new(JxlDecoder::new(reader)?))),
        );
        image::hooks::register_format_detection_hook("jxl".into(), CODESTREAM, None);
        image::hooks::register_format_detection_hook("jxl".into(), CONTAINER, None);
    });
}
