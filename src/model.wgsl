
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
    [[location(0)]] real_position: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.projection * camera.view * camera.model * vec4<f32>(model.position, 1.0);
    out.real_position = out.position.xyz;
    return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    //solarized theme, factor out into separate bind group later
    let key = vec3<f32>(0.99, 0.96, 0.89);
    let fill = vec3<f32>(0.92, 0.91, 0.83);
    let base = vec3<f32>(0.20, 0.24, 0.25);
    //The shading of the fragment should depend on the relative angle from the z-normal vector. 
    // This simulates a "light" emanating from the camera and from the upper right.
    
    //we obtain the normal vector by taking the cross product of the partial derivative of position w.r.t. x and y
    //the final color is a function of the angle between the normal vector and reference vectors. 
    var n_screen: vec3<f32> = cross( dpdx(in.real_position), dpdy(in.real_position));
    n_screen.z = n_screen.z * camera.projection[2][2]; //this component of the projection matrix is the zoom level
    let normal = normalize(n_screen);

    //determine projection of fragment normal vector onto z unit vector
    let a = dot(normal, vec3<f32>(0.0, 0.0, -1.0));
    let b = dot(normal, vec3<f32>(-0.57, -0.57, 0.57));

    return vec4<f32>(mix(base, key, a)* 0.3 + mix(base, fill, b) * 0.7, 1.0);
}