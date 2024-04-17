use std::sync::atomic::AtomicBool;

use sdl2::sys::{SDL_Renderer, SDL_Texture};

use crate::sdl2;

use super::util::{constants::SCREEN_WIDTH, kinds::{Camera, Sectors, Walls}};
pub struct State {
    pub window: *mut sdl2::video::Window,
    pub renderer: *mut SDL_Renderer,
    pub texture: *mut SDL_Texture,
    pub debug: *mut SDL_Texture,
    pub pixels: *mut u32,
    pub quit: AtomicBool,

    pub sectors: Sectors,
    pub walls: Walls,

    pub y_lo: [u16; SCREEN_WIDTH],
    pub y_hi: [u16; SCREEN_WIDTH],

    pub camera: Camera,

    pub sleepy: bool
}