use std::{fs, hash::Hash};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::time::SystemTime;

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

pub struct Loader {
    pub filename: String,
    // pub vertex_set: HashMap<Vertex, usize>
    pub start_time: SystemTime,
}

impl Loader {

    fn parse_ascii(&self, stream: String) -> (Vec<Vertex>, Vec<u32>) {
        let floats: Vec<f32> = stream
        .split_ascii_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();
        
        let mut vertices = Vec::with_capacity(floats.len()/12);
        let mut indices = Vec::with_capacity(floats.len()/12);
        let mut i = 0;
        for triangle in floats.chunks(12) {
            for vertex in triangle[3..].chunks(3) {
                vertices.push(Vertex { pos: vertex.try_into().expect("Slice with incorrect length")});
                indices.push(i);
                i += 1;
            }
        }

        (vertices, indices)
    }


    pub fn run(&self) -> (Vec<Vertex>, Vec<u32>){
        
        let bytestream = fs::read(&self.filename).unwrap();
        if bytestream.len() < 84 {
            panic!("File is too small to be an STL file: ({} < 84 bytes)", bytestream.len());
        }
        let result = match std::str::from_utf8(&bytestream[0..5]) {
            Ok(header) if header == "solid" => {
                self.parse_ascii(String::from_utf8_lossy(bytestream.as_slice()).to_string())
            }
            _ => {
                self.parse_binary(bytestream)
            }
        };
        let parse_time = SystemTime::now();
        let dt = parse_time.duration_since(self.start_time).expect("Negative parse time calculated?");
        println!("Time to parse files {:?}", dt);
        result
    }

    pub fn parse_binary(&self, bytestream: Vec<u8>) -> (Vec<Vertex>, Vec<u32>) {
        
        //not sure if this approach is better than the byteorder approach, which requires a mutable borrow 
        //(and will be difficult to use in a multithreaded context.)
        let num_triangles = u32::from_le_bytes(bytestream[80..84].try_into().expect("Slice with incorrect length")); 

        //TODO: enable multithreading
        let body = &bytestream[84..];
        
        // let mut triangle_data = Vec::new();
        let mut vertex_data = Vec::with_capacity(num_triangles as usize*3);
        let mut indices = Vec::with_capacity(num_triangles as usize *3);
        let i = 0;
        //loop over every 50 chunks. The first 36 bytes are vertex data. 
        for triangle_data in body.chunks(50) {
            for n in 1..4 {
                let mut vertex = Vertex {pos: [0.0;3]};
                for (mut data, val) in triangle_data.chunks(4).skip(n*3).zip(vertex.pos.iter_mut()) {
                    *val = data.read_f32::<LittleEndian>().unwrap();
                }
                vertex_data.push(vertex);
                indices.push(i as u32);
            }

            //last 2 bytes are the "attribute byte count" and are ignored.
        }

        // println!("{:?}", vertex_data);
        // println!("{}", indices.last().unwrap());

        (vertex_data, indices)
    }

    
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::time::SystemTime;

    use super::{Loader, Vertex};

    #[test]
    fn test_binary_load() {
        let filename = "models/cube.stl".to_string();

        let bytestream = fs::read(&filename).unwrap();
        let loader = Loader {filename, start_time: SystemTime::now()};
        let (vertices, _) = loader.parse_binary(bytestream);
        let ans = vec![
            Vertex { pos: [-35.0, 60.0, 20.0] },
            Vertex { pos: [-55.0, 60.0, 20.0] },
            Vertex { pos: [-35.0, 40.0, 20.0] },
            Vertex { pos: [-35.0, 40.0, 20.0] },
            Vertex { pos: [-55.0, 60.0, 20.0] },
            Vertex { pos: [-55.0, 40.0, 20.0] },
            Vertex { pos: [-35.0, 40.0, 0.0] },
            Vertex { pos: [-55.0, 40.0, 0.0] },
            Vertex { pos: [-35.0, 60.0, 0.0] },
            Vertex { pos: [-35.0, 60.0, 0.0] },
            Vertex { pos: [-55.0, 40.0, 0.0] },
            Vertex { pos: [-55.0, 60.0, 0.0] },
            Vertex { pos: [-55.0, 40.0, 20.0] },
            Vertex { pos: [-55.0, 40.0, 0.0] },
            Vertex { pos: [-35.0, 40.0, 20.0] },
            Vertex { pos: [-35.0, 40.0, 20.0] },
            Vertex { pos: [-55.0, 40.0, 0.0] },
            Vertex { pos: [-35.0, 40.0, 0.0] },
            Vertex { pos: [-55.0, 60.0, 20.0] },
            Vertex { pos: [-55.0, 60.0, 0.0] },
            Vertex { pos: [-55.0, 40.0, 20.0] },
            Vertex { pos: [-55.0, 40.0, 20.0] },
            Vertex { pos: [-55.0, 60.0, 0.0] },
            Vertex { pos: [-55.0, 40.0, 0.0] },
            Vertex { pos: [-35.0, 60.0, 20.0] },
            Vertex { pos: [-35.0, 60.0, 0.0] },
            Vertex { pos: [-55.0, 60.0, 20.0] },
            Vertex { pos: [-55.0, 60.0, 20.0] },
            Vertex { pos: [-35.0, 60.0, 0.0] },
            Vertex { pos: [-55.0, 60.0, 0.0] },
            Vertex { pos: [-35.0, 40.0, 20.0] },
            Vertex { pos: [-35.0, 40.0, 0.0] },
            Vertex { pos: [-35.0, 60.0, 20.0] },
            Vertex { pos: [-35.0, 60.0, 20.0] },
            Vertex { pos: [-35.0, 40.0, 0.0] },
            Vertex { pos: [-35.0, 60.0, 0.0] }
        ];
        assert_eq!(ans.len(), vertices.len());
        assert_eq!(vertices, ans);
    }
}