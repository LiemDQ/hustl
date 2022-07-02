use std::{fs, hash::Hash};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32;3],
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

pub struct Loader {
    pub filename: String,
    // pub vertex_set: HashMap<Vertex, usize>
}

impl Loader {

    fn parse_ascii(&self) -> Vec<Vertex> {
        let stream = fs::read_to_string(&self.filename).unwrap();
        todo!();

    }

    pub fn run(&self, ){
        
    }

    pub fn parse_binary(&self) -> (Vec<Vertex>, Vec<u32>) {
        // let file = std::fs::File::create(&self.filename).unwrap();
        let bytestream = fs::read(&self.filename).unwrap();
        if bytestream.len() < 84 {
            panic!("File is too small to be an STL file: ({} < 84 bytes)", bytestream.len());
        }
        //not sure if this approach is better than the byteorder approach, which requires a mutable borrow 
        //(and will be difficult to use in a multithreaded context.)
        let num_triangles = u32::from_le_bytes(bytestream[80..84].try_into().expect("Slice with incorrect length")); 

        //TODO: enable multithreading
        let body = &bytestream[84..];
        let body_iter = body.chunks(50); 
        // let mut triangle_data = Vec::new();
        let mut vertex_data = Vec::with_capacity(num_triangles as usize*3);
        let mut indices = Vec::with_capacity(num_triangles as usize *3);

        for (i,vertex_vals) in body_iter.enumerate() {
            let mut vertex = Vertex {pos: [0.0;3]}; //initialized to 0 before filling. If this affects performance significantly can use unsafe initialization instead. 

            for (mut data, val) in vertex_vals.chunks(4).zip(vertex.pos.iter_mut()) {
                *val = data.read_f32::<LittleEndian>().unwrap();
                //last 2 bytes are the "attribute byte count" and are ignored.
            }
            vertex_data.push(vertex);
            indices.push(i as u32);
        }

        (vertex_data, indices)
    }

    
}