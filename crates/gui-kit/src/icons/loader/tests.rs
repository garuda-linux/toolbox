use std::sync::Arc;
use std::time::{Duration, Instant};

use egui::load::{ImageLoadResult, ImageLoader as _, ImagePoll, LoadError, SizeHint};

use super::{THEME_URI, ThemeLoader};
use crate::icons::{GO_HOME, ThemeIcons};

const PATIENCE: Duration = Duration::from_secs(10);

const SQUARE: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M1 1h14v14H1z"/></svg>"#;

const HINT: SizeHint = SizeHint::Size {
    width: 22,
    height: 22,
    maintain_aspect_ratio: true,
};

fn theme_uri(name: &str) -> String {
    format!("{THEME_URI}Papirus/{name}.svg")
}

fn context_with(uri: &str, bytes: &'static [u8]) -> egui::Context {
    let ctx = egui::Context::default();
    ctx.include_bytes(uri.to_owned(), bytes);
    ctx
}

fn settled(loader: &ThemeLoader, ctx: &egui::Context, uri: &str) -> ImageLoadResult {
    let deadline = Instant::now() + PATIENCE;
    loop {
        let result = loader.load(ctx, uri, HINT);
        if !matches!(result, Ok(ImagePoll::Pending { size: _ })) || Instant::now() > deadline {
            return result;
        }

        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn a_theme_file_renders_off_the_frame_and_comes_back_ready() {
    let uri = theme_uri("go-home");
    let ctx = context_with(&uri, SQUARE);
    let loader = ThemeLoader::default();

    assert!(matches!(
        loader.load(&ctx, &uri, HINT),
        Ok(ImagePoll::Pending { size: _ })
    ));

    let Ok(ImagePoll::Ready { image }) = settled(&loader, &ctx, &uri) else {
        panic!("the theme file did not render");
    };

    assert_eq!(image.size, [22, 22]);
    assert!(!loader.has_pending());
}

#[test]
fn bundled_icons_are_left_to_egui_extras() {
    let uri = "bytes://icons/bundled/go-home.svg";
    let ctx = context_with(uri, SQUARE);

    assert!(matches!(
        ThemeLoader::default().load(&ctx, uri, HINT),
        Err(LoadError::NotSupported)
    ));
}

#[test]
fn a_broken_theme_file_fails_without_stopping_the_worker() {
    let broken = theme_uri("broken");
    let good = theme_uri("go-home");
    let ctx = context_with(&broken, b"<svg");
    ctx.include_bytes(good.clone(), SQUARE);
    let loader = ThemeLoader::default();

    assert!(matches!(
        settled(&loader, &ctx, &broken),
        Err(LoadError::Loading(_))
    ));
    assert!(matches!(
        settled(&loader, &ctx, &good),
        Ok(ImagePoll::Ready { image: _ })
    ));
}

#[test]
fn forgetting_a_theme_renders_its_files_again() {
    let uri = theme_uri("go-home");
    let ctx = context_with(&uri, SQUARE);
    let loader = Arc::new(ThemeLoader::default());
    assert!(matches!(
        settled(&loader, &ctx, &uri),
        Ok(ImagePoll::Ready { image: _ })
    ));

    loader.forget_all();

    assert!(matches!(
        loader.load(&ctx, &uri, HINT),
        Ok(ImagePoll::Pending { size: _ })
    ));
}

/// The body-text size the icon widgets draw at, as `icon_image` sizes it.
#[test]
fn warming_renders_the_body_text_size_before_any_widget_asks() {
    let harness = crate::testing::harness::<()>(100.0, 100.0).build_ui(|_| {});
    let ctx = &harness.ctx;
    let mut theme = ThemeIcons::new("Papirus");
    theme.insert(GO_HOME.name(), Arc::from(SQUARE));
    crate::icons::use_theme(ctx, Some(Arc::new(theme)));

    super::warm(ctx);

    let body = egui::TextStyle::Body.resolve(&ctx.global_style());
    let edge = ctx.fonts_mut(|fonts| fonts.row_height(&body)) + crate::ui_ext::ICON_OVER_TEXT;
    let hint = egui::ImageSize {
        maintain_aspect_ratio: true,
        max_size: egui::Vec2::INFINITY,
        fit: egui::ImageFit::Exact(egui::vec2(edge, edge)),
    }
    .hint(egui::Vec2::INFINITY, ctx.pixels_per_point());

    let uri = theme_uri(GO_HOME.name());

    let deadline = Instant::now() + PATIENCE;
    while ctx.has_pending_images() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }

    assert!(matches!(
        ctx.try_load_image(&uri, hint),
        Ok(ImagePoll::Ready { image: _ })
    ));
}
