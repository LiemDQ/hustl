use std::{fs};
use byteorder::{LittleEndian, ReadBytesExt};


type Triangle = [f32; 12];
enum ParserState {
    Start,
    ModelSize,
    GPUBuffer,
    WorkerGPU,
    Done,
    Error,
    ErrorNoFile,
    ErrorBadASCII,
    ErrorWrongSize,
}
pub struct Parser {
    pub filename: String,
}

impl Parser {

    fn parse_ascii(&self) -> Vec<Triangle> {
        let stream = fs::read_to_string(&self.filename).unwrap();
        todo!();

    }

    pub fn run(&self, ){
        
    }

    pub fn parse_binary(&self) -> Vec<Triangle> {
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
        let mut triangle_data = Vec::with_capacity(num_triangles as usize);
        for triangle_vals in body_iter {
            let mut triangle: Triangle = [0.;12]; //initialized to 0 before filling. If this affects performance significantly can use unsafe initialization instead. 

            for (mut data, val) in triangle_vals.chunks(4).zip(triangle.iter_mut()) {
                *val = data.read_f32::<LittleEndian>().unwrap();
                //last 2 bytes are the "attribute byte count" and are ignored.
            }
            triangle_data.push(triangle);
        }

        triangle_data
    }

    
}