
use super::constants;
use crate::res::util::kinds::V2;

pub fn deg_2_rad(d: f32) -> f32 {
    d * (constants::PI / 180.0)
}

pub fn rad_2_deg(r: f32) -> f32 {
    r * (180.0 / constants::PI)
}

pub fn dot(u: V2, v: V2) -> f32 {
    (u.x * v.x) + (u.y * v.y)
}

pub fn length(v: V2) -> f32 {
    f32::sqrt(dot(v.clone(), v)) 
}