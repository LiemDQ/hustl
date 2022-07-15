use std::{fs, hash::Hash};
use ahash::AHashMap;
use crossbeam::thread::ScopedJoinHandle;
use std::time::SystemTime;
use std::thread;


const BYTES_PER_TRIANGLE: u32 = 50;

///includes normal vector, which isn't used in our implementation
const FLOATS_PER_TRIANGLE: u32 = 12; 

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

#[derive(Debug, Clone)]
pub struct ModelBounds {
    pub x: (f32, f32), //min, max
    pub y: (f32, f32), //min, max
    pub z: (f32, f32), //min, max
}

impl ModelBounds {
    fn merge_bounds(&mut self, other: &ModelBounds) {
        self.x.0 = self.x.0.min(other.x.0);
        self.x.1 = self.x.1.max(other.x.1);
        self.y.0 = self.y.0.min(other.y.0);
        self.y.1 = self.y.1.max(other.y.1);
        self.z.0 = self.z.0.min(other.z.0);
        self.z.1 = self.z.1.max(other.z.1);
    }

    fn update(&mut self, vertex: &Vertex) {
        self.x.0 = self.x.0.min(vertex.pos[0]);
        self.x.1 = self.x.1.max(vertex.pos[0]);
        self.y.0 = self.y.0.min(vertex.pos[1]);
        self.y.1 = self.y.1.max(vertex.pos[1]);
        self.z.0 = self.z.0.min(vertex.pos[2]);
        self.z.1 = self.z.1.max(vertex.pos[2]);
    }
}

impl Default for ModelBounds {
    fn default() -> Self {
        //we want any value for min or max to override the default value. 
        Self {
            x: (f32::INFINITY, f32::NEG_INFINITY),
            y: (f32::INFINITY, f32::NEG_INFINITY),
            z: (f32::INFINITY, f32::NEG_INFINITY),
        }
    }
}

pub struct ModelData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounds: ModelBounds,
}

pub struct Loader {
    pub filename: String,
    pub start_time: SystemTime,
    pub max_workers: Option<usize>, 
}

impl Loader {
    pub fn new(filename: String, start_time: SystemTime, max_workers: Option<usize>) -> Self {
        Self { filename, start_time, max_workers}
    }

    fn parse_ascii(&self, stream: String) -> ModelData {
        let floats: Vec<f32> = stream
        .split_ascii_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

        let num_triangles = (floats.len()/12) as u32;
        let num_threads = if let Some(max_workers) = self.max_workers {
            max_workers.min(thread::available_parallelism().expect("Could not query number of cores").get()) as u32
        } else {
            thread::available_parallelism().expect("Could not query number of cores").get() as u32
        };
        println!("Number of loaders: {}", num_threads);
        let triangles_per_thread = num_triangles/num_threads;

        let float_slice = floats.as_slice();

        crossbeam::scope(move |s| {
            let handles: Vec<crossbeam::thread::ScopedJoinHandle<_>> = (0..num_threads).map(|n| {
                let mut worker = Worker::new(n, triangles_per_thread);
                if n == num_threads - 1 {
                    let starting_float = (FLOATS_PER_TRIANGLE*triangles_per_thread*n) as usize;
                    s.spawn( move |_| {
                        worker.run_ascii(&float_slice[starting_float..])
                    })
                } else {
                    let starting_float = (FLOATS_PER_TRIANGLE*triangles_per_thread*n) as usize;
                    let ending_float = (FLOATS_PER_TRIANGLE*triangles_per_thread*(n+1)) as usize;
                    s.spawn ( move |_| {
                        worker.run_ascii(&float_slice[starting_float..ending_float])
                    })
                }
            }).collect();

            //with vector indexing we roughly estimate number of entries is divided by 6
            let vertex_data = Vec::with_capacity(num_triangles as usize/2);
            let indices = Vec::with_capacity(num_triangles as usize *3);
            
            Loader::process_workers(handles.into_iter(), vertex_data, indices)
        }).unwrap()        
    }


    pub fn run(&self) -> ModelData {
        
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

    fn parse_binary(&self, bytestream: Vec<u8>) -> ModelData {
        
        //not sure if this approach is better than the byteorder approach, which requires a mutable borrow 
        //(and will be difficult to use in a multithreaded context.)
        let num_triangles = u32::from_le_bytes(bytestream[80..84].try_into().expect("Slice with incorrect length")); 

        //TODO: enable multithreading
        let num_threads = if let Some(max_workers) = self.max_workers {
            max_workers.min(thread::available_parallelism().expect("Could not query number of cores").get()) as u32
        } else {
            thread::available_parallelism().expect("Could not query number of cores").get() as u32
        };
        println!("Number of loaders: {}", num_threads);
        let triangles_per_thread = num_triangles/num_threads;
        let remaining_triangles = num_triangles % num_threads;
        //number of bytes per worker: 50 bytes/triangle * triangles_per_thread
        
        let body = &bytestream[84..];
        
        crossbeam::scope(move |s| {
            let handles: Vec<crossbeam::thread::ScopedJoinHandle<_>> = (0..num_threads).map(|n| {
                let mut worker = Worker::new(n, triangles_per_thread);
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

            //with vector indexing we roughly estimate number of entries is divided by 6
            let vertex_data = Vec::with_capacity(num_triangles as usize/2);
            let indices = Vec::with_capacity(num_triangles as usize *3);

            Loader::process_workers(handles.into_iter(), vertex_data, indices)
        }).unwrap()
    } 

    fn process_workers<'scope, T>(handles: T, mut vertices: Vec<Vertex>, mut indices: Vec<u32>) -> ModelData 
        where T: Iterator<Item = ScopedJoinHandle<'scope, ModelData>>
    {
        let mut current_index: u32 = 0;
        let mut bounds = ModelBounds::default();

        for handle in handles {
            let data = handle.join().unwrap();

            vertices.extend(&data.vertices);
            
            //the index numbers need to be offset based on how many entries are currently in the vertex_data vector, 
            //since they start from 0.
            indices.extend(data.indices.iter().map(|idx| *idx + current_index)); 
            bounds.merge_bounds(&data.bounds);
            current_index = vertices.len() as u32;
        }

        ModelData { vertices, indices, bounds}
    }
}

/// Loader worker
/// Worker id is a value between 0 and X, where X is the maximum number of threads. 
struct Worker {
    vertex_map: AHashMap<Vertex, u32>,
    id: u32, 
    triangles_per_worker: u32
}

impl Worker {

    pub fn new(id: u32, triangles_per_worker: u32) -> Self {
        Self {vertex_map: AHashMap::default(), id, triangles_per_worker}
    }

    pub fn run_binary(&mut self, bytes: &[u8], n: u32) -> ModelData {
        self.get_binary_vertices_indexed(bytes, n)
    }

    pub fn run_ascii(&mut self, floats: &[f32]) -> ModelData {
        self.get_ascii_vertices_indexed(floats)
    }
    
    #[allow(dead_code)]
    fn get_binary_vertices_unindexed(&self, bytes: &[u8], n: u32) -> ModelData {
        let mut i = 0;
        let mut vertices = Vec::with_capacity(n as usize);
        let mut indices = Vec::with_capacity(n as usize);
        let mut bounds = ModelBounds::default();

        for triangle_data in bytes.chunks(50) {
            for n in 1..4 {
                let mut vertex = Vertex {pos: [0.0;3]};
                for (data, val) in triangle_data.chunks(4).skip(n*3).zip(vertex.pos.iter_mut()) {
                    *val = f32::from_le_bytes(data.try_into().expect("Slice with incorrect length"));
                }
                bounds.update(&vertex);
                vertices.push(vertex);
                indices.push(i);
                i += 1;
            }
            //last 2 bytes are the "attribute byte count" and are ignored.
        }
        ModelData { vertices, indices, bounds }
    }

    fn get_binary_vertices_indexed(&mut self, bytes: &[u8], n: u32) -> ModelData {
        let mut vertices = Vec::with_capacity(n as usize*3);
        let mut indices = Vec::with_capacity(n as usize *3);
        let mut bounds = ModelBounds::default();
        //loop over every 50 chunks. The first 36 bytes are vertex data. 
        for chunk in bytes.chunks(50) {
            
            for n in 1..4 {
                let mut vertex = Vertex {pos: [0.0;3]};
                for (data, val) in chunk.chunks(4).skip(n*3).zip(vertex.pos.iter_mut()) {
                    *val = f32::from_le_bytes(data.try_into().expect("Slice with incorrect length"));
                }
                bounds.update(&vertex);
                let idx = self.get_vertex_index(vertex, &mut vertices);
                indices.push(idx);
            }
           //last 2 bytes are the "attribute byte count" and are ignored.   
        }

        ModelData { vertices, indices, bounds }
    }

    #[allow(dead_code)]
    fn get_ascii_vertices_unindexed(&self, floats: &[f32]) -> ModelData {
        let mut vertices = Vec::with_capacity(floats.len()/12*3);
        let mut indices = Vec::with_capacity(floats.len()/12*3);
        let mut i = 0;
        let mut bounds = ModelBounds::default();

        for triangle in floats.chunks(12) {
            for vertex in triangle[3..].chunks(3) {
                let vertex = Vertex { pos: vertex.try_into().expect("Slice with incorrect length")};
                bounds.update(&vertex);
                vertices.push(vertex);
                indices.push(i);
                i += 1;
            }
        }

        ModelData { vertices, indices, bounds }
    }

    fn get_ascii_vertices_indexed(&mut self, floats: &[f32]) -> ModelData {
        let mut vertices = Vec::with_capacity(floats.len()/12*3);
        let mut indices = Vec::with_capacity(floats.len()/12*3);
        let mut bounds = ModelBounds::default();

        for triangle in floats.chunks(12) {
            for vertex in triangle[3..].chunks(3) {
                let vertex = Vertex { pos: vertex.try_into().expect("Slice with incorrect length")};
                bounds.update(&vertex);
                let idx = self.get_vertex_index(
                    vertex,
                    &mut vertices
                );
                indices.push(idx)
            }
        }

        ModelData { vertices, indices, bounds }
    }

    fn get_vertex_index(&mut self, vertex: Vertex, vector: &mut Vec<Vertex>) -> u32 {
        if let Some(idx) = self.vertex_map.get(&vertex) {
            *idx
        } else {
            vector.push(vertex);
            let idx = (vector.len() -1) as u32;
            self.vertex_map.insert(vertex, idx);
            idx
        }
    }

    fn calculate_index(&self, idx: u32) -> u32 {
        idx + self.id*self.triangles_per_worker*3
    }
    
    #[allow(dead_code)]
    fn calculate_starting_index(&self) -> u32 {
        self.calculate_index(0)
    }
}

    

#[cfg(test)]
mod test {
    use std::fs;
    use std::time::SystemTime;

    use super::{Loader, Vertex, BYTES_PER_TRIANGLE, Worker};

    const CUBE_VERTICES: [Vertex; 36] = [
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

    const ASCII_CUBE_VERTICES: [Vertex;36] = [
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

    const CUBE_VERTICES_DEDUPLICATED: [Vertex; 8] = [
        Vertex { pos: [-35.0, 60.0, 20.0] }, 
        Vertex { pos: [-55.0, 60.0, 20.0] }, 
        Vertex { pos: [-35.0, 40.0, 20.0] }, 
        Vertex { pos: [-55.0, 40.0, 20.0] }, 
        Vertex { pos: [-35.0, 40.0, 0.0] }, 
        Vertex { pos: [-55.0, 40.0, 0.0] }, 
        Vertex { pos: [-35.0, 60.0, 0.0] }, 
        Vertex { pos: [-55.0, 60.0, 0.0] }
    ];

    const CUBE_INDICES: [u32; 36] = [
        0, 1, 2, 
        2, 1, 3, 
        4, 5, 6, 
        6, 5, 7, 
        3, 5, 2,
        2, 5, 4, 
        1, 7, 3, 
        3, 7, 5, 
        0, 6, 1, 
        1, 6, 7, 
        2, 4, 0, 
        0, 4, 6
    ];

    const ASCII_CUBE_VERTICES_DEDUPLICATED: [Vertex; 8] = [
        Vertex { pos: [0.0, 0.0, 10.0] },   //0
        Vertex { pos: [10.0, 0.0, 10.0] },  //1
        Vertex { pos: [0.0, 10.0, 10.0] },  //2
        Vertex { pos: [10.0, 10.0, 10.0] }, //3
        Vertex { pos: [10.0, 0.0, 0.0] },   //4
        Vertex { pos: [10.0, 10.0, 0.0] },  //5
        Vertex { pos: [0.0, 0.0, 0.0] },    //6
        Vertex { pos: [0.0, 10.0, 0.0] }    //7
    ];

    const ASCII_CUBE_INDICES: [u32; 36] = [
        0, 1, 2, 
        3, 2, 1,
        1, 4, 3,
        5, 3, 4,
        4, 6, 5,
        7, 5, 6,
        6, 0, 7,
        2, 7, 0,
        2, 3, 7,
        5, 7, 3,
        1, 0, 4, 
        6, 4, 0,
    ];

    #[test]
    fn test_binary_load() {
        let filename = "assets/cube.stl".to_string();

        let bytestream = fs::read(&filename).unwrap();
        let loader = Loader::new(filename,SystemTime::now(), Some(1));
        let data = loader.parse_binary(bytestream);
        let ans = &CUBE_VERTICES_DEDUPLICATED[..];
        
        assert_eq!(data.vertices, ans);
        assert_eq!(data.indices, &CUBE_INDICES[..]);
    }

    #[test]
    fn test_ascii_load() {
        let filename = "assets/cube-ascii.stl".to_string();
        let stream = fs::read_to_string(&filename).unwrap();
        let loader = Loader::new(filename, SystemTime::now(), Some(1));
        let data = loader.parse_ascii(stream);
        let ans = &ASCII_CUBE_VERTICES_DEDUPLICATED[..];
        assert_eq!(data.vertices, ans);
        assert_eq!(data.indices, &ASCII_CUBE_INDICES[..]);
    }

    #[test]
    fn test_loader_run(){
        let filename = "assets/cube.stl".to_string();
        let loader = Loader::new(filename,SystemTime::now(), Some(1));
        let data = loader.run();

        assert_eq!(data.vertices, &CUBE_VERTICES_DEDUPLICATED[..]);
        assert_eq!(data.indices, &CUBE_INDICES[..]);
    }

    #[test]
    fn test_binary_unindexed_worker(){
        let filename = "assets/cube.stl".to_string();
        let bytestream = fs::read(&filename).unwrap();
        let bytes = &bytestream[84..];
        let num_triangles = bytes.len() as u32/BYTES_PER_TRIANGLE ;
        let worker = Worker::new(0, num_triangles);
        let data = worker.get_binary_vertices_unindexed(bytes, num_triangles*3);

        assert_eq!(data.vertices, &CUBE_VERTICES[..]);
    }

    #[test]
    fn test_binary_indexed_worker(){
        let filename = "assets/cube.stl".to_string();
        let bytestream = fs::read(&filename).unwrap();
        let bytes = &bytestream[84..];
        let num_triangles = bytes.len() as u32/BYTES_PER_TRIANGLE ;
        let mut worker = Worker::new(0, num_triangles);
        let data = worker.get_binary_vertices_indexed(bytes, num_triangles*3);

        assert_eq!(data.vertices, &CUBE_VERTICES_DEDUPLICATED[..]);
        assert_eq!(data.indices, &CUBE_INDICES[..]);
    }

    #[test]
    fn test_ascii_unindexed_worker(){
        let filename = "assets/cube-ascii.stl".to_string();
        let stream = fs::read_to_string(&filename).unwrap();
        let floats: Vec<f32> = stream
        .split_ascii_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

        let num_triangles = (floats.len()/12) as u32;

        let worker = Worker::new(0, num_triangles);
        let data = worker.get_ascii_vertices_unindexed(floats.as_slice());
        let ans = &ASCII_CUBE_VERTICES[..];

        assert_eq!(data.vertices, ans);
    }

    #[test]
    fn test_ascii_indexed_worker(){
        let filename = "assets/cube-ascii.stl".to_string();
        let stream = fs::read_to_string(&filename).unwrap();
        let floats: Vec<f32> = stream
        .split_ascii_whitespace()
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

        let num_triangles = (floats.len()/12) as u32;

        let mut worker = Worker::new(0, num_triangles);
        let data = worker.get_ascii_vertices_indexed(floats.as_slice());
        let ans = &ASCII_CUBE_VERTICES_DEDUPLICATED[..];

        assert_eq!(data.vertices, ans);
        assert_eq!(data.indices, &ASCII_CUBE_INDICES[..]);
    }
}