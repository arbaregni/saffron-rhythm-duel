#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

// we can import items from shader modules in the assets folder with a quoted path
// #import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> start_time: f32;
@group(2) @binding(2) var<uniform> duration: f32;


const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.5);

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = (globals.time - start_time) / duration;

    // set up the alpha (actually an alpha multiplier)
    var alpha = 1.0;

    // fade to 100% to 0% alpha over time
    alpha *= (1.0 - t);

    // flip the y-axis and center on the x
    let st = vec2f(
        2.0 * in.uv[0] - 1.0,
        1.0 - in.uv[1]
    );

    // fade out at the top
    alpha *= pow(1.0 - st[1], 2.0);

    // fade out in the middle
    // alpha *= (0.9 - pow(abs(st[0]), 1.0));

    // expand rectangle outwards over time
    let extents = saturate(1.0 - pow(2.0, -10.0 * t));
    let mask = step(-extents, st[0])
             * step(st[0], extents);

    // make it completely invisible outside mask
    alpha = mix(0.0, alpha, mask);
    
    return vec4<f32>(material_color[0], material_color[1], material_color[2], material_color[3] * alpha);
}
