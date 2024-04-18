#[derive(Debug, Clone)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct V2i {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct Wall {
    pub a: V2i,
    pub b: V2i,
    pub portal: usize
}

#[derive(Debug, Clone)]
pub struct Walls {
    pub arr: [Wall; 128],
    pub n: usize,
}

#[derive(Debug, Clone)]
pub struct Sector {
    pub id: i32,
    pub firstwall: usize,
    pub nwalls: usize,
    pub zfloor: f32,
    pub zceil: f32,
}

#[derive(Debug, Clone)]
pub struct Sectors {
    pub arr: [Sector; 32],
    pub n: usize,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: V2,
    pub angle: f32,
    pub anglecos: f32,
    pub anglesin: f32,
    pub sector: i32,
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