struct CameraUniform {
    view: mat4x4<f32>;
    model: mat4x4<f32>;
    projection: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.projection * camera.view * camera.model * vec4<f32>(model.position, 1.0);
    return out;
}



[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    //the shading of the fragment should depend on the relative angle from the z-normal vector.
    //we obtain the normal vector by taking the cross product of the partial derivative w.r.t. x and y
    //the final color is a function of the angle between the normal vector and reference vectors. 
    var n_screen: vec4<f32> = cross(dpdx(in.position), dpdy(in.position));
    // n_screen.z = n_screen.z * 
    let normal = normalize(n_screen);

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}