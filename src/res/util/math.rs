use super::constants::{self, HFOV, PI, PI_2, PI_4, SCREEN_WIDTH, TAU};
use crate::res::{game::State, util::kinds::V2};

#[inline]
pub fn deg_2_rad(d: f32) -> f32 {
    d * (constants::PI / 180.0)
}

#[inline]
pub fn rad_2_deg(r: f32) -> f32 {
    r * (180.0 / constants::PI)
}

#[inline]
pub fn dot(u: V2, v: V2) -> f32 {
    (u.x * v.x) + (u.y * v.y)
}

#[inline]
pub fn length(v: V2) -> f32 {
    f32::sqrt(dot(v.clone(), v))
}

#[inline]
pub fn normalize(v: V2) -> V2 {
    let m = length(v.clone());
    assert!(m != 0.0);
    V2::new(v.x / m, v.y / m)
}

#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
pub fn clamp<T: PartialOrd>(x: T, min_val: T, max_val: T) -> T {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}

#[inline]
pub fn ifnan(x: f32, alt: f32) -> f32 {
    if x.is_nan() {
        alt
    } else {
        x
    }
}

#[inline]
pub fn point_side(p: V2, a: V2, b: V2) -> f32 {
    -((p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x))
}

#[inline]
pub fn rotate(v: V2, a: f32) -> V2 {
    V2 {
        x: (v.x * a.cos()) - (v.y * a.sin()),
        y: (v.x * a.sin()) + (v.y * a.cos()),
    }
}

#[inline]
pub fn intersect_segs(a0: V2, a1: V2, b0: V2, b1: V2) -> V2 {
    let d: f32 = ((a0.x - a1.x) * (b0.y - b1.y)) - ((a0.y - a1.y) * (b0.x - b1.x));

    if d.abs() < 0.000001 {
        return V2 {
            x: f32::NAN,
            y: f32::NAN,
        };
    }

    let t: f32 = (((a0.x - b0.x) * (b0.y - b1.y)) - ((a0.y - b0.y) * (b0.x - b1.x))) / d;

    let u: f32 = (((a0.x - b0.x) * (a0.y - a1.y)) - ((a0.y - b0.y) * (a0.x - a1.x))) / d;

    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        V2 {
            x: a0.x + t * (a1.x - a0.x),
            y: a0.y + t * (a1.y - a0.y),
        }
    } else {
        V2 {
            x: f32::NAN,
            y: f32::NAN,
        }
    }
}

#[inline]
pub fn abgr_mul(col: u32, a: u32) -> u32 {
    let br: u32 = ((col & 0xFF00FF) * a) >> 8;
    let g: u32 = ((col & 0x00FF00) * a) >> 8;

    return 0xFF000000 | (br & 0xFF00FF) | (g & 0x00FF00);
}

#[inline]
pub fn screen_angle_to_x(angle: f32) -> i32 {
    return ((SCREEN_WIDTH / 2) as i32)
        * (1.0 - f32::tan(((angle + (HFOV / 2.0)) / HFOV) * PI_2 - PI_4)) as i32;
}

#[inline]
pub fn normalize_angle(a: f32) -> f32 {
    return a - (TAU * f32::floor((a + PI) / TAU));
}

#[inline]
pub fn world_pos_to_camera(p: V2, state: State) -> V2 {
    let u: V2 = V2::new(p.x - state.camera.pos.x, p.y - state.camera.pos.y);
    return V2::new(
        u.x * state.camera.anglesin - u.y * state.camera.anglecos,
        u.x * state.camera.anglecos + u.y * state.camera.anglesin,
    );
}
