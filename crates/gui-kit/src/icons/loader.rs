//! Renders the desktop theme's SVG files on a worker thread. `egui_extras`
//! renders an SVG inside the frame that first draws it, at every new size,
//! and a theme file is outside text of any shape.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, OnceLock};

use egui::load::{Bytes, BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint};
use egui::{ColorImage, Context, ImageFit, ImageSize, TextStyle, Vec2, vec2};
use parking_lot::Mutex;

/// Where [`super::Icon::image`] puts a theme's file. The bundled glyphs
/// have their own prefix and stay with `egui_extras`: they are small, ours,
/// and must show in the first frame.
pub(super) const THEME_URI: &str = "bytes://icons/theme/";

type Key = (String, SizeHint);

enum Slot {
    Rendering,
    Done {
        result: Result<Arc<ColorImage>, String>,
        used: u64,
    },
}

#[derive(Default)]
struct State {
    pass: u64,
    slots: HashMap<Key, Slot>,
}

struct Job {
    key: Key,
    bytes: Bytes,
    ctx: Context,
}

#[derive(Default)]
pub struct ThemeLoader {
    state: Arc<Mutex<State>>,
    worker: OnceLock<Sender<Job>>,
}

impl ThemeLoader {
    pub const ID: &'static str = egui::generate_loader_id!(ThemeLoader);

    fn send(&self, job: Job) {
        let worker = self.worker.get_or_init(|| spawn(Arc::clone(&self.state)));
        if let Err(failed) = worker.send(job) {
            self.state.lock().slots.insert(
                failed.0.key,
                Slot::Done {
                    result: Err("the icon worker stopped".to_owned()),
                    used: 0,
                },
            );
        }
    }
}

impl ImageLoader for ThemeLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if !uri.starts_with(THEME_URI) {
            return Err(LoadError::NotSupported);
        }

        let key = (uri.to_owned(), size_hint);
        {
            let mut state = self.state.lock();
            let pass = state.pass;
            match state.slots.get_mut(&key) {
                Some(Slot::Rendering) => return Ok(ImagePoll::Pending { size: None }),
                Some(Slot::Done { result, used }) => {
                    *used = pass;
                    return match result {
                        Ok(image) => Ok(ImagePoll::Ready {
                            image: Arc::clone(image),
                        }),
                        Err(error) => Err(LoadError::Loading(error.clone())),
                    };
                }
                None => {}
            }
        }

        match ctx.try_load_bytes(uri) {
            Ok(BytesPoll::Ready {
                bytes,
                size: _,
                mime: _,
            }) => {
                self.state.lock().slots.insert(key.clone(), Slot::Rendering);
                self.send(Job {
                    key,
                    bytes,
                    ctx: ctx.clone(),
                });
                Ok(ImagePoll::Pending { size: None })
            }
            Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
            Err(error) => Err(error),
        }
    }

    fn forget(&self, uri: &str) {
        self.state.lock().slots.retain(|(key, _), _| key != uri);
    }

    fn forget_all(&self) {
        self.state.lock().slots.clear();
    }

    /// As in `egui_extras`: a file drawn at several sizes keeps only those
    /// of the last pass, so a zoom does not pile up renders.
    fn end_pass(&self, pass_index: u64) {
        let mut state = self.state.lock();
        state.pass = pass_index;

        let mut sizes: HashMap<String, usize> = HashMap::new();
        for (uri, _) in state.slots.keys() {
            *sizes.entry(uri.clone()).or_default() += 1;
        }

        state.slots.retain(|(uri, _), slot| match slot {
            Slot::Rendering => true,
            Slot::Done { used, result: _ } => {
                sizes.get(uri).is_none_or(|count| *count < 2)
                    || pass_index <= used.saturating_add(1)
            }
        });
    }

    fn byte_size(&self) -> usize {
        self.state
            .lock()
            .slots
            .values()
            .map(|slot| match slot {
                Slot::Done {
                    result: Ok(image),
                    used: _,
                } => image.pixels.len() * size_of::<egui::Color32>(),
                Slot::Done { result: _, used: _ } | Slot::Rendering => 0,
            })
            .sum()
    }

    fn has_pending(&self) -> bool {
        self.state
            .lock()
            .slots
            .values()
            .any(|slot| matches!(slot, Slot::Rendering))
    }
}

/// Starts rendering the theme in use at the sizes the widgets draw it.
///
/// The frames after a theme change then rarely show an empty icon. A size
/// missed here still renders on the worker, one frame later.
pub fn warm(ctx: &Context) {
    let Some(theme) = super::theme_in_use(ctx) else {
        return;
    };

    let body = TextStyle::Body.resolve(&ctx.global_style());
    let line = ctx.fonts_mut(|fonts| fonts.row_height(&body));
    let points = [
        line + crate::ui_ext::ICON_OVER_TEXT,
        crate::widgets::buttons::GLYPH,
        crate::widgets::buttons::SMALL_GLYPH,
        crate::widgets::removable::GLYPH,
    ];

    for (name, file) in &theme.files {
        let uri = format!("{THEME_URI}{}/{name}.svg", theme.theme);
        ctx.include_bytes(uri.clone(), Arc::clone(&file.svg));
        for edge in points {
            let size = ImageSize {
                maintain_aspect_ratio: true,
                max_size: Vec2::INFINITY,
                fit: ImageFit::Exact(vec2(edge, edge)),
            };

            let hint = size.hint(Vec2::INFINITY, ctx.pixels_per_point());
            if let Err(error) = ctx.try_load_image(&uri, hint) {
                tracing::debug!("cannot warm {uri}: {error}");
            }
        }
    }
}

/// One thread for every file, so a theme change renders one icon after
/// the other instead of all at once.
fn spawn(state: Arc<Mutex<State>>) -> Sender<Job> {
    let (sender, jobs) = mpsc::channel::<Job>();
    let started = std::thread::Builder::new()
        .name("theme-icons".to_owned())
        .spawn(move || {
            let options = resvg::usvg::Options::default();
            for Job { key, bytes, ctx } in jobs {
                let result = egui_extras::image::load_svg_bytes_with_size(&bytes, key.1, &options)
                    .map(Arc::new);

                if let Err(error) = &result {
                    tracing::info!("the theme's {} cannot be drawn: {error}", key.0);
                }

                let mut state = state.lock();
                let used = state.pass;
                state.slots.insert(key, Slot::Done { result, used });
                drop(state);
                ctx.request_repaint();
            }
        });

    if let Err(error) = started {
        tracing::warn!("cannot start the icon worker: {error}");
    }

    sender
}
