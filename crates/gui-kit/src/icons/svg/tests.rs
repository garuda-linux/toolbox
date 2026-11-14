use std::path::Path;

use resvg::usvg::{Group, Node, Tree};

use super::options;

fn images(tree: &Tree) -> usize {
    count(tree.root())
}

fn count(group: &Group) -> usize {
    group
        .children()
        .iter()
        .map(|node| match node {
            Node::Image(_) => 1,
            Node::Group(inner) => count(inner),
            Node::Path(_) | Node::Text(_) => 0,
        })
        .sum()
}

/// The file exists, so the default resolver would read it.
#[test]
fn an_image_href_to_a_local_file_reads_nothing() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icons/applications-games.svg");
    assert!(file.exists(), "{}", file.display());
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="16" height="16"><image width="16" height="16" xlink:href="{}"/></svg>"#,
        file.display()
    );

    let tree = Tree::from_str(&svg, &options()).unwrap();
    assert_eq!(images(&tree), 0);

    let default = Tree::from_str(&svg, &resvg::usvg::Options::default()).unwrap();
    assert_eq!(images(&default), 1, "the default reads the file");
}

#[test]
fn an_image_in_a_data_url_is_left_out() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="16" height="16"><image width="16" height="16" xlink:href="data:image/png;base64,iVBORw0KGgo="/></svg>"#;

    let tree = Tree::from_str(svg, &options()).unwrap();
    assert_eq!(images(&tree), 0);
}

#[test]
fn the_window_parses_svgs_with_these_options_only() {
    let ctx = egui::Context::default();
    crate::install(&ctx);

    assert!(ctx.is_loader_installed(super::SvgLoader::ID));
    assert!(!ctx.is_loader_installed(egui_extras::loaders::svg_loader::SvgLoader::ID));
}
