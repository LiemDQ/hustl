struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>,
    [[location(0)]] normal: vec4<f32>,
};

[[stage(vertex)]]
fn vs_main(){
    
}

[[stage(fragment)]]
fn fs_main() {
    //the shading of the fragment should depend on the relative angle from the z-normal vector.
}