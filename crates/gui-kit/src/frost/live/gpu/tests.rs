use super::{Pipeline, texels};

#[test]
fn a_side_that_is_not_a_number_or_empty_has_no_texels() {
    assert_eq!(texels(f32::NAN), None);
    assert_eq!(texels(0.0), None);
    assert_eq!(texels(-3.0), None);
}

#[test]
fn a_side_takes_whole_texels_up_to_the_largest() {
    assert_eq!(texels(0.2), Some(1));
    assert_eq!(texels(10.5), Some(11));
    assert_eq!(texels(f32::INFINITY), Some(4096));
}

#[test]
fn a_shader_the_driver_refuses_leaves_no_pipeline() {
    let state = egui_kittest::wgpu::create_render_state(
        egui_kittest::wgpu::default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );

    assert!(Pipeline::with_shader(&state, "this is no shader").is_none());
    assert!(Pipeline::with_shader(&state, include_str!("../blur.wgsl")).is_some());
}

#[test]
fn a_texture_the_renderer_lacks_is_left_out_and_the_render_is_incomplete() {
    use egui::epaint::{ClippedPrimitive, Mesh, Primitive};
    use egui::{Rect, TextureId};

    let mesh = |texture| ClippedPrimitive {
        clip_rect: Rect::EVERYTHING,
        primitive: Primitive::Mesh(Mesh::with_texture(texture)),
    };

    let known = TextureId::Managed(0);
    let new = TextureId::Managed(20);

    let (kept, complete) = super::drawable(vec![mesh(known), mesh(new)], |id| *id == known);
    assert_eq!(kept.len(), 1);
    assert!(!complete);

    let (all, whole) = super::drawable(vec![mesh(known)], |id| *id == known);
    assert_eq!(all.len(), 1);
    assert!(whole);
}
