use sdl2::video::WindowContext;
use std::f32::consts::{FRAC_PI_4, PI};
use std::fs::File;
use std::io::{BufRead, BufReader};
#[derive(Clone, Copy)]
struct V2 {
    x: f32,
    y: f32,
}

impl Default for V2 {
    fn default() -> Self {
        V2 { x: 0.0, y: 0.0 }
    }
}

struct V2i {
    x: i32,
    y: i32,
}

impl Default for V2i {
    fn default() -> Self {
        V2i { x: 0, y: 0 }
    }
}

struct Wall {
    a: V2i,
    b: V2i,
    portal: i32,
}

struct Sector {
    id: i32,
    firstwall: usize,
    nwalls: usize,
    zfloor: f32,
    zceil: f32,
}

struct State<'a> {
    sdl_context: sdl2::Sdl,
    window: sdl2::video::Window,
    renderer: sdl2::render::WindowCanvas,
    texture: sdl2::render::Texture<'a>,
    debug: sdl2::render::Texture<'a>,
    pixels: Vec<u32>,
    quit: bool,
    sectors: Vec<Sector>,
    walls: Vec<Wall>,
    y_lo: Vec<u16>,
    y_hi: Vec<u16>,
    camera: Camera,
    sleepy: bool,
}

struct Camera {
    pos: V2,
    angle: f32,
    anglecos: f32,
    anglesin: f32,
    sector: i32,
}

const SCREEN_WIDTH: usize = 384;
const SCREEN_HEIGHT: usize = 216;
const EYE_Z: f32 = 1.65;
const HFOV: f32 = 90.0;
const VFOV: f32 = 0.5;
const ZNEAR: f32 = 0.0001;
const ZFAR: f32 = 128.0;

const PI_2: f32 = PI / 2.0;
const PI_4: f32 = PI / 4.0;

macro_rules! min {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;
        if a < b {
            a
        } else {
            b
        }
    }};
}

macro_rules! max {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;
        if a > b {
            a
        } else {
            b
        }
    }};
}

macro_rules! clamp {
    ($x:expr, $mi:expr, $ma:expr) => {
        min!(max!($x, $mi), $ma)
    };
}

struct QueueEntry {
    id: i32,
    x0: usize,
    x1: usize,
}

impl Default for QueueEntry {
    fn default() -> Self {
        QueueEntry {
            id: 0,
            x0: 0,
            x1: 0,
        }
    }
}

impl<'a> State<'a> {
    fn new(
        sdl_context: sdl2::Sdl,
        texture_creator: &'a sdl2::render::TextureCreator<WindowContext>,
    ) -> Result<State, String> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("raycast", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let renderer = video_subsystem
            .window("raycast", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string()).unwrap()
            .into_canvas()
            .build()
            .expect("Couldn't create window");

        let texture: sdl2::render::Texture<'_>;
        let debug: sdl2::render::Texture<'_>;
        texture = texture_creator
            .create_texture_streaming(
                Some(sdl2::pixels::PixelFormatEnum::RGBA8888),
                SCREEN_WIDTH as u32,
                SCREEN_HEIGHT as u32,
            )
            .expect("error creating textures");
        debug = texture_creator
            .create_texture_streaming(
                Some(sdl2::pixels::PixelFormatEnum::RGBA8888),
                SCREEN_WIDTH as u32,
                SCREEN_HEIGHT as u32,
            )
            .expect("error creating debug textures");

        let pixels = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];
        let y_lo = vec![0; SCREEN_WIDTH];
        let y_hi = vec![SCREEN_HEIGHT as u16 - 1; SCREEN_WIDTH];

        let camera = Camera {
            pos: V2 { x: 3.0, y: 3.0 },
            angle: 0.0,
            anglecos: 0.0,
            anglesin: 0.0,
            sector: 1,
        };

        Ok(State {
            sdl_context,
            window,
            renderer,
            texture,
            debug,
            pixels,
            quit: false,
            sectors: Vec::new(),
            walls: Vec::new(),
            y_lo,
            y_hi,
            camera,
            sleepy: false,
        })
    }

    fn load_sectors(&mut self, path: &str) -> Result<(), String> {
        // sector 0 does not exist
        self.sectors.push(Sector {
            id: 0,
            firstwall: 0,
            nwalls: 0,
            zfloor: 0.0,
            zceil: 0.0,
        });

        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        let mut ss = ScanState::None;

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                match line {
                    "[SECTOR]" => ss = ScanState::Sector,
                    "[WALL]" => ss = ScanState::Wall,
                    _ => return Err(format!("Unknown section: {}", line)),
                }
            } else {
                match ss {
                    ScanState::Wall => {
                        let mut iter = line.split_whitespace();
                        let a_x: i32 = iter
                            .next()
                            .ok_or("Expected a_x")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;
                        let a_y: i32 = iter
                            .next()
                            .ok_or("Expected a_y")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;
                        let b_x: i32 = iter
                            .next()
                            .ok_or("Expected b_x")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;
                        let b_y: i32 = iter
                            .next()
                            .ok_or("Expected b_y")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;
                        let portal: i32 = iter
                            .next()
                            .ok_or("Expected portal")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;

                        self.walls.push(Wall {
                            a: V2i { x: a_x, y: a_y },
                            b: V2i { x: b_x, y: b_y },
                            portal,
                        });
                    }
                    ScanState::Sector => {
                        let mut iter = line.split_whitespace();
                        let id: i32 = iter
                            .next()
                            .ok_or("Expected id")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;
                        let firstwall: usize = iter
                            .next()
                            .ok_or("Expected firstwall")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;
                        let nwalls: usize = iter
                            .next()
                            .ok_or("Expected nwalls")?
                            .parse()
                            .map_err(|e: std::num::ParseIntError| e.to_string())?;
                        let zfloor: f32 = iter
                            .next()
                            .ok_or("Expected zfloor")?
                            .parse()
                            .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                        let zceil: f32 = iter
                            .next()
                            .ok_or("Expected zceil")?
                            .parse()
                            .map_err(|e: std::num::ParseFloatError| e.to_string())?;

                        self.sectors.push(Sector {
                            id,
                            firstwall,
                            nwalls,
                            zfloor,
                            zceil,
                        });
                    }
                    ScanState::None => return Err("Unexpected data outside section".to_string()),
                }
            }
        }

        Ok(())
    }

    fn tick(&mut self) -> Result<(), String> {
        self.handle_input();

        if !self.sleepy {
            self.cast_rays();
            self.render();
        }

        Ok(())
    }

    fn handle_input(&mut self) {
        let event_pump = self.sdl_context.event_pump().unwrap();
        let keyboard_state = sdl2::keyboard::KeyboardState::new(&event_pump);
        let keys: Vec<sdl2::keyboard::Scancode> = keyboard_state.pressed_scancodes().collect();

        for key in keys {
            match key {
                sdl2::keyboard::Scancode::Escape => self.quit = true,
                sdl2::keyboard::Scancode::W => self.move_forward(),
                sdl2::keyboard::Scancode::S => self.move_backward(),
                sdl2::keyboard::Scancode::A => self.strafe_left(),
                sdl2::keyboard::Scancode::D => self.strafe_right(),
                sdl2::keyboard::Scancode::Left => self.rotate_left(),
                sdl2::keyboard::Scancode::Right => self.rotate_right(),
                sdl2::keyboard::Scancode::Up => self.look_up(),
                sdl2::keyboard::Scancode::Down => self.look_down(),
                _ => {}
            }
        }
    }

    fn move_forward(&mut self) {
        self.camera.pos.x += self.camera.anglecos * 0.1;
        self.camera.pos.y += self.camera.anglesin * 0.1;
        self.clip_movement();
    }

    fn move_backward(&mut self) {
        self.camera.pos.x -= self.camera.anglecos * 0.1;
        self.camera.pos.y -= self.camera.anglesin * 0.1;
        self.clip_movement();
    }

    fn strafe_left(&mut self) {
        self.camera.pos.x += self.camera.anglesin * 0.1;
        self.camera.pos.y -= self.camera.anglecos * 0.1;
        self.clip_movement();
    }

    fn strafe_right(&mut self) {
        self.camera.pos.x -= self.camera.anglesin * 0.1;
        self.camera.pos.y += self.camera.anglecos * 0.1;
        self.clip_movement();
    }

    fn rotate_left(&mut self) {
        self.camera.angle -= PI / 180.0;
        self.update_camera();
    }

    fn rotate_right(&mut self) {
        self.camera.angle += PI / 180.0;
        self.update_camera();
    }

    fn look_up(&mut self) {
        if self.camera.angle + FRAC_PI_4 < PI / 2.0 {
            self.camera.angle += FRAC_PI_4;
            self.update_camera();
        }
    }

    fn look_down(&mut self) {
        if self.camera.angle - FRAC_PI_4 > 0.0 {
            self.camera.angle -= FRAC_PI_4;
            self.update_camera();
        }
    }

    fn clip_movement(&mut self) {
        let sector_id = self.camera.sector as usize;
        let sector = &self.sectors[sector_id];
        let pos = &mut self.camera.pos;

        pos.x = clamp!(
            pos.x,
            self.walls[sector.firstwall].a.x as f32,
            self.walls[sector.firstwall + sector.nwalls - 1].a.x as f32
        );
        pos.y = clamp!(
            pos.y,
            self.walls[sector.firstwall].a.y as f32,
            self.walls[sector.firstwall + sector.nwalls - 1].a.y as f32
        );
    }

    fn update_camera(&mut self) {
        self.camera.anglecos = self.camera.angle.cos();
        self.camera.anglesin = self.camera.angle.sin();
    }

    fn cast_rays(&mut self) {
        // Implementation of ray casting logic goes here
        // ...

        // For now, let's just fill the screen with a color
        for pixel in self.pixels.iter_mut() {
            *pixel = 0xFF_0000FF; // Red
        }
    }

    fn render(&mut self) {
        // Implementation of rendering logic goes here
        // ...

        // For now, update the texture with the pixel data
        self.texture
            .update(
                None,
                &self
                    .pixels
                    .iter()
                    .flat_map(|&p| p.to_ne_bytes())
                    .collect::<Vec<u8>>(),
                SCREEN_WIDTH * 4,
            )
            .unwrap();
        self.renderer.clear();
        self.renderer.copy(&self.texture, None, None).unwrap();
        self.renderer.present();
    }
}

enum ScanState {
    None,
    Sector,
    Wall,
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("raycast", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let renderer = window
        .into_canvas()
        .build()
        .expect("Couldn't create window");

    let texture_creator = renderer.texture_creator();
    let mut state = State::new(sdl_context, &texture_creator)?;
    state.load_sectors("sectors.txt")?;

    while !state.quit {
        state.tick()?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
