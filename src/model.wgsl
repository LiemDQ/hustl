
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
    //solarized theme, factor out into separate bind gorup later
    let key = vec3<f32>(253.0/255.0, 246.0/255.0, 227.0/255.0);
    let fill = vec3<f32>(216.0/255.0, 222.0/255.0, 233.0/255.0);
    let base = vec3<f32>(76.0/255.0, 86.0/255.0, 106.0/255.0);
    //the shading of the fragment should depend on the relative angle from the z-normal vector.
    //we obtain the normal vector by taking the cross product of the partial derivative w.r.t. x and y
    //the final color is a function of the angle between the normal vector and reference vectors. 
    var n_screen: vec3<f32> = cross(dpdx(in.position.xyz), dpdy(in.position.xyz));
    n_screen.z = n_screen.z * camera.projection[2][2];
    let normal = normalize(n_screen);
    let a = dot(normal, vec3<f32>(0.0, 0.0, 1.0));
    let b = dot(normal, vec3<f32>(-0.57, -0.57, 0.57));

    return vec4<f32>(mix(base, key, a)* 0.5 + mix(base, fill, b) * 0.5, 1.0);
}