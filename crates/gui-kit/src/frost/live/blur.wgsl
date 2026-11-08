struct Step { step: vec2<f32>, pad: vec2<f32> };
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear: sampler;
@group(0) @binding(2) var<uniform> u: Step;

struct Out { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> Out {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    var o: Out;
    o.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    o.uv = vec2<f32>(x, y);
    return o;
}

@fragment
fn fs(v: Out) -> @location(0) vec4<f32> {
    let w0 = 0.2270270270;
    let w1 = 0.3162162162;
    let w2 = 0.0702702703;
    let o1 = 1.3846153846;
    let o2 = 3.2307692308;
    var c = textureSample(source, linear, v.uv) * w0;
    c += textureSample(source, linear, v.uv + u.step * o1) * w1;
    c += textureSample(source, linear, v.uv - u.step * o1) * w1;
    c += textureSample(source, linear, v.uv + u.step * o2) * w2;
    c += textureSample(source, linear, v.uv - u.step * o2) * w2;
    return c;
}
