//use super::math::deg_2_rad;
//use lazy_static::lazy_static;
use std::f32::consts::PI;


pub const TAU: f32 = 2.0 * PI;

pub const PI_2: f32 = PI / 2.0;

pub const PI_4: f32 = PI / 4.0;

pub const SCREEN_WIDTH: usize = 384;


pub const SCREEN_HEIGHT: i32 = 216;

pub const EYE_Z: f32 = 1.65;

pub const HFOV: f32 = std::f32::consts::FRAC_PI_2;


pub const VFOV: f32 = 0.5;

pub const ZNEAR: f32 = 0.0001;

pub const ZFAR: f32 = 128.0;

pub const SECTOR_NON: i32 = 0;

pub const SECTOR_MAX: i32 = 128;

pub const QUEUE_MAX: i32 = 64;

