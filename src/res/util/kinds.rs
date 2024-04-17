#[derive(Clone)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct V2i {
    pub x: i32,
    pub y: i32,
}

impl V2 {
    pub fn new(x: f32, y: f32) -> Self {
        V2 {x, y}
    }

    pub fn v2_to_v2i(self) -> V2i {
        V2i::new(self.x as i32, self.y as i32)
    }
}

impl V2i {
    pub fn new(x: i32, y:i32) -> Self {
        V2i {x, y}
    }

    
    pub fn v2i_to_v2(self) -> V2 {
        V2::new(self.x as f32, self.y as f32)
    }
}