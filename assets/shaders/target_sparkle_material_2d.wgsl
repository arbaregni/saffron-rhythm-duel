#import bevy_sprite::mesh2d_vertex_output::VertexOutput
// we can import items from shader modules in the assets folder with a quoted path
#import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var base_color_texture: texture_2d<f32>;
@group(2) @binding(2) var base_color_sampler: sampler;

const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.5);

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let r: f32 = distance(mesh.uv, vec2<f32>(0.5)) * 1.4;
    let alpha: f32 = pow(r, 2.0);
    return vec4<f32>(material_color[0], material_color[1], material_color[2], alpha);
    // return material_color * textureSample(base_color_texture, base_color_sampler, mesh.uv) * COLOR_MULTIPLIER;
}
