use std::{fs, hash::Hash};
use byteorder::{LittleEndian, ReadBytesExt};
use fxhash::FxHashMap;
use ahash::AHashMap;
use std::time::SystemTime;
use std::thread;


const BYTES_PER_TRIANGLE: u32 = 50;

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
        state.write_i32(bytemuck::cast::<f32, i32>(self.pos[0]));
        state.write_i32(bytemuck::cast::<f32, i32>(self.pos[1]));
        state.write_i32(bytemuck::cast::<f32, i32>(self.pos[2]));
    }
}

pub struct Loader {
    pub filename: String,
    pub vertex_map: FxHashMap<Vertex, usize>,
    // pub vertex_set: HashMap<Vertex, usize>
    pub start_time: SystemTime,
}

impl Loader {
    pub fn new(filename: String, start_time: SystemTime) -> Self {
        Self { filename, vertex_map: FxHashMap::default(), start_time}
    }

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
        let num_threads = thread::available_parallelism().expect("Could not query threads").get() as u32;
        println!("Number of loaders: {}", num_threads);
        let triangles_per_thread = num_triangles/num_threads;
        let remaining_triangles = num_triangles % num_threads;
        //number of bytes per worker: 50 bytes/triangle * triangles_per_thread
        
        let body = &bytestream[84..];
        
        crossbeam::scope(move |s| {
            let handles: Vec<crossbeam::thread::ScopedJoinHandle<_>> = (0..num_threads).map(|n| {
                let worker = Worker::new(n, triangles_per_thread);
                if n == num_threads - 1 {
                    let starting_byte = (BYTES_PER_TRIANGLE*triangles_per_thread*n) as usize;
                    s.spawn( move |_| {
                        worker.run_binary(&body[starting_byte..], (triangles_per_thread+ remaining_triangles)*3)
                    })
                } else {
                    let starting_byte = (BYTES_PER_TRIANGLE*triangles_per_thread*n) as usize;
                    let ending_byte = (BYTES_PER_TRIANGLE*triangles_per_thread*(n+1)) as usize;
                    s.spawn ( move |_| {
                        worker.run_binary(&body[starting_byte..ending_byte], triangles_per_thread*3)
                    })
                }
            }).collect();

            let mut vertex_data = Vec::with_capacity(num_triangles as usize*3);
            let mut indices = Vec::with_capacity(num_triangles as usize *3);

            for handle in handles {
                let (data, idx) = handle.join().unwrap();
                vertex_data.extend(&data);
                indices.extend(&idx);
            }

            (vertex_data, indices)
        }).unwrap()
    }

    fn insert_into_map(&mut self, vertex: Vertex, vector: &mut Vec<Vertex>) -> u32 {
        if let Some(idx) = self.vertex_map.get(&vertex) {
            *idx as u32
        } else {
            vector.push(vertex);
            let idx = vector.len() -1;
            self.vertex_map.insert(vertex, idx);
            idx as u32
        }
    }    
}

/// Loader worker
/// Worker id is a value between 0 and X, where X is the maximum number of threads. 
struct Worker {
    vertex_map: AHashMap<Vertex, u32>,
    id: u32, 
    triangles_per_thread: u32
}

impl Worker {

    pub fn new(id: u32, triangles_per_thread: u32) -> Self {
        Self {vertex_map: AHashMap::default(), id, triangles_per_thread}
    }

    pub fn run_binary(&self, bytes: &[u8], n: u32) -> (Vec<Vertex>, Vec<u32>) {
        self.get_vertices_unindexed(bytes, n)
    }

    fn insert_into_map(&mut self, vertex: Vertex, vector: &mut Vec<Vertex>) -> u32 {
        if let Some(idx) = self.vertex_map.get(&vertex) {
            *idx
        } else {
            vector.push(vertex);
            let idx = self.calculate_index((vector.len() -1) as u32);
            self.vertex_map.insert(vertex, idx);
            idx
        }
    }

    fn get_vertices_unindexed(&self, bytes: &[u8], n: u32) -> (Vec<Vertex>, Vec<u32>) {
        let mut i = self.calculate_starting_index();
        let mut vertex_data = Vec::with_capacity(n as usize);
        let mut indices = Vec::with_capacity(n as usize);

        for triangle_data in bytes.chunks(50) {
            for n in 1..4 {
                let mut vertex = Vertex {pos: [0.0;3]};
                for (data, val) in triangle_data.chunks(4).skip(n*3).zip(vertex.pos.iter_mut()) {
                    *val = f32::from_le_bytes(data.try_into().expect("Slice with incorrect length"));
                }
                vertex_data.push(vertex);
                indices.push(i);
                i += 1;
            }
            //last 2 bytes are the "attribute byte count" and are ignored.
        }
        (vertex_data, indices)
    }

    fn get_vertices_indexed(&self, bytes: &[u8], n: u32) -> (Vec<Vertex>, Vec<u32>) {
        todo!("vvertex indexing")
    }

    fn calculate_index(&self, idx: u32) -> u32 {
        idx + self.id*self.triangles_per_thread*3
    }

    fn calculate_starting_index(&self) -> u32 {
        self.calculate_index(0)
    }
}

    

#[cfg(test)]
mod test {
    use std::fs;
    use std::time::SystemTime;

    use super::{Loader, Vertex};
    

    #[test]
    fn test_binary_load() {
        let filename = "assets/cube.stl".to_string();

        let bytestream = fs::read(&filename).unwrap();
        let loader = Loader::new(filename,SystemTime::now());
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

    #[test]
    fn test_ascii_load() {
        let filename = "assets/cube-ascii.stl".to_string();
        let stream = fs::read_to_string(&filename).unwrap();
        let loader = Loader::new(filename, SystemTime::now());
        let (vertices, _) = loader.parse_ascii(stream);
        let ans = vec![
            Vertex { pos: [0.0, 0.0, 10.0] },
            Vertex { pos: [10.0, 0.0, 10.0] },
            Vertex { pos: [0.0, 10.0, 10.0] },

            Vertex { pos: [10.0, 10.0, 10.0] },
            Vertex { pos: [0.0, 10.0, 10.0] },
            Vertex { pos: [10.0, 0.0, 10.0] },

            Vertex { pos: [10.0, 0.0, 10.0] },
            Vertex { pos: [10.0, 0.0, 0.0] },
            Vertex { pos: [10.0, 10.0, 10.0] },

            Vertex { pos: [10.0, 10.0, 0.0] },
            Vertex { pos: [10.0, 10.0, 10.0] },
            Vertex { pos: [10.0, 0.0, 0.0] },

            Vertex { pos: [10.0, 0.0, 0.0] },
            Vertex { pos: [0.0, 0.0, 0.0] },
            Vertex { pos: [10.0, 10.0, 0.0] },

            Vertex { pos: [0.0, 10.0, 0.0] },
            Vertex { pos: [10.0, 10.0, 0.0] },
            Vertex { pos: [0.0, 0.0, 0.0] },

            Vertex { pos: [0.0, 0.0, 0.0] },
            Vertex { pos: [0.0, 0.0, 10.0] },
            Vertex { pos: [0.0, 10.0, 0.0] },

            Vertex { pos: [0.0, 10.0, 10.0] },
            Vertex { pos: [0.0, 10.0, 0.0] },
            Vertex { pos: [0.0, 0.0, 10.0] },

            Vertex { pos: [0.0, 10.0, 10.0] },
            Vertex { pos: [10.0, 10.0, 10.0] },
            Vertex { pos: [0.0, 10.0, 0.0] },

            Vertex { pos: [10.0, 10.0, 0.0] },
            Vertex { pos: [0.0, 10.0, 0.0] },
            Vertex { pos: [10.0, 10.0, 10.0] },

            Vertex { pos: [10.0, 0.0, 10.0] },
            Vertex { pos: [0.0, 0.0, 10.0] },
            Vertex { pos: [10.0, 0.0, 0.0] },

            Vertex { pos: [0.0, 0.0, 0.0] },
            Vertex { pos: [10.0, 0.0, 0.0] },
            Vertex { pos: [0.0, 0.0, 10.0] }
        ];
        assert_eq!(ans.len(), vertices.len());
        assert_eq!(ans, vertices);
    }
}