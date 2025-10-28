use ultraviolet::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Box3D {
    max_vertex: Vec3,
    min_vertex: Vec3,
}

impl Box3D {
    pub fn new(max_vertex: Vec3, min_vertex: Vec3) -> Box3D {
        return Self {
            max_vertex: max_vertex,
            min_vertex: min_vertex,
        };
    }

    #[inline]
    pub fn colide(&self, r#box: &Box3D) -> bool {
        self.max_vertex.x >= r#box.min_vertex.x
            && self.min_vertex.x <= r#box.max_vertex.x
            && self.max_vertex.y >= r#box.min_vertex.y
            && self.min_vertex.y <= r#box.max_vertex.y
            && self.max_vertex.z >= r#box.min_vertex.z
            && self.min_vertex.z <= r#box.max_vertex.z
    }

    pub fn transformation(&mut self, mat: Mat4) {
        let corners = [
            Vec3::new(self.min_vertex.x, self.min_vertex.y, self.min_vertex.z),
            Vec3::new(self.min_vertex.x, self.min_vertex.y, self.max_vertex.z),
            Vec3::new(self.min_vertex.x, self.max_vertex.y, self.min_vertex.z),
            Vec3::new(self.min_vertex.x, self.max_vertex.y, self.max_vertex.z),
            Vec3::new(self.max_vertex.x, self.min_vertex.y, self.min_vertex.z),
            Vec3::new(self.max_vertex.x, self.min_vertex.y, self.max_vertex.z),
            Vec3::new(self.max_vertex.x, self.max_vertex.y, self.min_vertex.z),
            Vec3::new(self.max_vertex.x, self.max_vertex.y, self.max_vertex.z),
        ];

        let mut min_vertex = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_vertex = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for c in &corners {
            let t = mat * Vec4::new(c.x, c.y, c.z, 1.0); // w = 1

            self.min_vertex.x = min_vertex.x.min(t.x);
            self.min_vertex.y = min_vertex.y.min(t.y);
            self.min_vertex.z = min_vertex.z.min(t.z);

            self.max_vertex.x = max_vertex.x.max(t.x);
            self.max_vertex.y = max_vertex.y.max(t.y);
            self.max_vertex.z = max_vertex.z.max(t.z);
        }
    }

    #[inline]
    pub fn move_delta(&mut self, delta: Vec3) {
        self.min_vertex += delta;
        self.max_vertex += delta;
    }
    //??????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????
    /*
    #[inline]
    pub fn change_position(&mut self, position: Vec3) {
        self.min_vertex = position;
        self.max_vertex = position;
    }
    */
}
