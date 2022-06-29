pub struct Mesh {
    vertices: Vec<f32>,
    indices: Vec<usize>,
}

impl Mesh {
    pub fn min(&self, start: usize) -> Option<f32> {
        if start >= self.vertices.len() {
            return None;
        }
        let mut v = self.vertices[start];
        for vnew in self.vertices.iter().skip(start).step_by(3) {
            v = f32::min(v, vnew);
        }
        Some(v)
    }

    pub fn empty(&self) -> bool {
        self.vertices.len() == 0
    }

    pub fn tri_count(&self) -> usize {
        self.indices.len()/3
    }
}