// Taken from Foxtrot GUI crate.
// https://github.com/Formlabs/foxtrot/blob/master/gui/src/backdrop.wgsl
// This shader allows us to achieve a nice color gradient in the background.
// Siple, but effective.

struct VertexOutput {
    [[location(0)]] color: vec4<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct BackgroundColors {
    corner1: vec4<f32>;
    corner2: vec4<f32>;
    corner3: vec4<f32>;
    corner4: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> colors: BackgroundColors;

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let c1 = colors.corner1;
    let c2 = colors.corner2;
    let c3 = colors.corner3;
    let c4 = colors.corner4;
    
    //draw two triangles to cover the entirety of the screen
    if (in_vertex_index == 0u || in_vertex_index == 5u) {
        out.color = c1;
        out.position = vec4<f32>(-1.0, -1.0, 0.0, 1.0);
    } else if (in_vertex_index == 1u) {
        out.color = c2;
        out.position = vec4<f32>(1.0, -1.0, 0.0, 1.0);
    } else if (in_vertex_index == 2u || in_vertex_index == 3u) {
        out.color = c3;
        out.position = vec4<f32>(1.0, 1.0, 0.0, 1.0);
    } else if (in_vertex_index == 4u) {
        out.color = c4;
        out.position = vec4<f32>(-1.0, 1.0, 0.0, 1.0);
    } else {
        out.color = c4;
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}
