use std::{fs, hash::Hash};
use byteorder::{LittleEndian, ReadBytesExt};
use fxhash::FxHashMap;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32;3],
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos[0] == other.pos[0] && self.pos[1] == other.pos[1] && self.pos[2] == other.pos[2]
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(bytemuck::cast::<f32, u32>(self.pos[0]));
        state.write_u32(bytemuck::cast::<f32, u32>(self.pos[1]));
        state.write_u32(bytemuck::cast::<f32, u32>(self.pos[2]));
    }
}

pub struct Loader {
    pub filename: String,
    pub vertex_map: FxHashMap<Vertex, usize>
}

impl Loader {
    pub fn new(filename: String) -> Self {
        Self { filename, vertex_map: FxHashMap::default() }
    }

    fn parse_ascii(&self) -> Vec<Vertex> {
        let stream = fs::read_to_string(&self.filename).unwrap();
        todo!();

    }

    pub async fn run(&self, ){
        
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
        
        // let mut triangle_data = Vec::new();
        let mut vertex_data = Vec::with_capacity(num_triangles as usize*3);
        let mut indices = Vec::with_capacity(num_triangles as usize *3);
        let mut i = 0;
        //loop over every 50 chunks. The first 36 bytes are vertex data. 
        body.chunks(50).map(|chunk| {
            
            for n in 1..4 {
                let mut vertex = Vertex {pos: [0.0;3]};
                for (mut data, val) in chunk.chunks(4).skip(n*3).zip(vertex.pos.iter_mut()) {
                    *val = data.read_f32::<LittleEndian>().unwrap();
                }
                
                vertex_data.push(vertex);
                indices.push(i as u32);
                i += 1;
            }
           //last 2 bytes are the "attribute byte count" and are ignored.
            
        }).collect();

        // println!("{:?}", vertex_data);
        // println!("{}", indices.last().unwrap());

        (vertex_data, indices)
    }

    fn insert_into_map(&mut self, vertex: Vertex, vector: &mut Vec<Vertex>) -> usize {
        if self.vertex_map.contains_key(&vertex) {
            *self.vertex_map.get(&vertex).unwrap()
        } else {
            vector.push(vertex);
            let idx = vector.len() -1;
            self.vertex_map.insert(vertex, idx);
            idx
        }
    }

    
}