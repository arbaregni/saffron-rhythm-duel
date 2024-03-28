#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_functions::{get_model_matrix, mesh2d_position_local_to_clip},
}
// we can import items from shader modules in the assets folder with a quoted path

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> created_at: f32;
@group(2) @binding(2) var<uniform> life_length: f32;


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // center it
    let uv = mesh.uv * 2.0 - 1.0;

    let t = (globals.time - created_at) / life_length;
    let alpha = 1.0 - t;

    return vec4<f32>(material_color.xyz, material_color.a * alpha); // vec4<f32>(uv, 0.0, 1.0);
}
