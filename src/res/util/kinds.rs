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

pub struct Wall {
    a: V2i,
    b: V2i,
    portal: i32
}
pub struct Walls {
    arr: [Wall; 128],
    n: usize,
}

pub struct Sector {
    id: i32,
    firstwall: usize,
    nwalls: usize,
    zfloor: f32,
    zceil: f32,
}

pub struct Sectors {
    arr: [Sector; 32],
    n: usize,
}

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