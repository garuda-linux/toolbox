//! Every SVG the window draws, bundled or from the icon theme or an app's
//! picture, is parsed with [`options`].
//!
//! usvg's default reads the file an `<image href>` names, so an icon in
//! `~/.local/share/icons` naming `/dev/zero` would exhaust memory and a
//! FIFO would hang the thread. The SVG loader of `egui_extras` keeps that
//! default, so [`SvgLoader`] stands in for it.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;

use egui::load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint};
use egui::{ColorImage, Context};
use parking_lot::Mutex;
use resvg::usvg::{ImageHrefResolver, Options};

struct Entry {
    used: u64,
    result: Result<Arc<ColorImage>, String>,
}

#[derive(Default)]
struct State {
    pass: u64,
    cache: HashMap<String, HashMap<SizeHint, Entry>>,
}

/// The SVG loader of `egui_extras` with [`options`].
pub struct SvgLoader {
    state: Mutex<State>,
    options: Options<'static>,
}

impl SvgLoader {
    pub const ID: &'static str = egui::generate_loader_id!(SvgLoader);

    fn cached(&self, uri: &str, size_hint: SizeHint) -> Option<ImageLoadResult> {
        let mut state = self.state.lock();
        let pass = state.pass;
        let entry = state.cache.get_mut(uri)?.get_mut(&size_hint)?;
        entry.used = pass;
        let result = entry.result.clone();
        drop(state);

        Some(match result {
            Ok(image) => Ok(ImagePoll::Ready { image }),
            Err(error) => Err(LoadError::Loading(error)),
        })
    }
}

impl Default for SvgLoader {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::default()),
            options: options(),
        }
    }
}

impl ImageLoader for SvgLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if !egui::load::has_extension(uri, "svg") {
            return Err(LoadError::NotSupported);
        }

        if let Some(cached) = self.cached(uri, size_hint) {
            return cached;
        }

        let bytes = match ctx.try_load_bytes(uri)? {
            BytesPoll::Ready {
                size: _,
                bytes,
                mime: _,
            } => bytes,
            BytesPoll::Pending { size } => return Ok(ImagePoll::Pending { size }),
        };

        let result = egui_extras::image::load_svg_bytes_with_size(&bytes, size_hint, &self.options)
            .map(Arc::new);

        let mut state = self.state.lock();
        let used = state.pass;
        state.cache.entry(uri.to_owned()).or_default().insert(
            size_hint,
            Entry {
                used,
                result: result.clone(),
            },
        );
        drop(state);

        result
            .map(|image| ImagePoll::Ready { image })
            .map_err(LoadError::Loading)
    }

    fn forget(&self, uri: &str) {
        self.state.lock().cache.remove(uri);
    }

    fn forget_all(&self) {
        self.state.lock().cache.clear();
    }

    fn byte_size(&self) -> usize {
        self.state
            .lock()
            .cache
            .values()
            .flat_map(HashMap::values)
            .map(|entry| match &entry.result {
                Ok(image) => image.pixels.len() * size_of::<egui::Color32>(),
                Err(error) => error.len(),
            })
            .sum()
    }

    /// Of an SVG drawn at several sizes only the sizes of the last frame
    /// stay, as a resized window would keep every size otherwise.
    fn end_pass(&self, pass_index: u64) {
        let mut state = self.state.lock();
        state.pass = pass_index;
        state.cache.retain(|_uri, bucket| {
            if bucket.len() >= 2 {
                bucket.retain(|_size, entry| pass_index <= entry.used + 1);
            }

            !bucket.is_empty()
        });
    }
}

/// No `<image>` of an SVG is read, neither from a file nor from a data
/// URL, so parsing touches nothing but the SVG's own bytes.
pub fn options() -> Options<'static> {
    Options {
        image_href_resolver: ImageHrefResolver {
            resolve_data: Box::new(|_mime, _data, _options| None),
            resolve_string: Box::new(|_href, _options| None),
        },
        ..Options::default()
    }
}
