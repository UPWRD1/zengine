use std::fs::File;
use std::io::{BufRead, BufReader};
use std::slice;

use super::util::constants::*;
use super::util::kinds::{Sector, V2i, Wall, V2};
use super::util::math::*;
use super::util::{
    constants::SCREEN_WIDTH,
    kinds::{Camera, Sectors, Walls},
};

use sdl2_sys::*;

#[derive(Debug, Clone)]
pub struct State {
    pub window: *mut SDL_Window,
    pub renderer: *mut SDL_Renderer,
    pub texture: *mut SDL_Texture,
    pub debug: *mut SDL_Texture,
    pub pixels: Vec<u32>,
    pub quit: bool,

    pub sectors: Sectors,
    pub walls: Walls,

    pub y_lo: [u16; SCREEN_WIDTH],
    pub y_hi: [u16; SCREEN_WIDTH],

    pub camera: Camera,

    pub sleepy: bool,
}

pub enum ScanState {
    ScanSector,
    ScanWall,
    ScanNone,
}

fn load_sectors(path: &str, state: &mut State) -> Result<(), i32> {
    // sector 0 does not exist
    state.sectors.n = 1;

    let f = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(-1),
    };
    let reader = BufReader::new(f);

    let mut retval = 0;
    let mut ss = ScanState::ScanNone;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                retval = -128;
                break;
            }
        };

        let mut p = line.trim_start();
        if p.is_empty() || p.starts_with('#') {
            continue;
        } else if p.starts_with('[') {
            p = p.trim_start_matches(|c| c == '[');
            let buf = [0; 64];
            if let Some(section) = p.split(']').next() {
                match section {
                    "SECTOR" => ss = ScanState::ScanSector,
                    "WALL" => ss = ScanState::ScanWall,
                    _ => {
                        retval = -3;
                        break;
                    }
                }
            } else {
                retval = -2;
                break;
            }
        } else {
            match ss {
                ScanState::ScanWall => {
                    if let Some(wall) = state.walls.arr.get_mut(state.walls.n) {
                        let parts: Vec<&str> = p.split_whitespace().collect();
                        if parts.len() != 5 {
                            retval = -4;
                            break;
                        }
                        if let (Ok(ax), Ok(ay), Ok(bx), Ok(by), Ok(portal)) = (
                            parts[0].parse::<i32>(),
                            parts[1].parse::<i32>(),
                            parts[2].parse::<i32>(),
                            parts[3].parse::<i32>(),
                            parts[4].parse::<i32>(),
                        ) {
                            wall.a.x = ax;
                            wall.a.y = ay;
                            wall.b.x = bx;
                            wall.b.y = by;
                            wall.portal = portal as usize;
                            state.walls.n += 1;
                        } else {
                            retval = -4;
                            break;
                        }
                    } else {
                        retval = -4;
                        break;
                    }
                }
                ScanState::ScanSector => {
                    if let Some(sector) = state.sectors.arr.get_mut(state.sectors.n) {
                        let parts: Vec<&str> = p.split_whitespace().collect();
                        if parts.len() != 5 {
                            retval = -5;
                            break;
                        }
                        if let (Ok(id), Ok(firstwall), Ok(nwalls), Ok(zfloor), Ok(zceil)) = (
                            parts[0].parse::<i32>(),
                            parts[1].parse::<usize>(),
                            parts[2].parse::<usize>(),
                            parts[3].parse::<f32>(),
                            parts[4].parse::<f32>(),
                        ) {
                            sector.id = id;
                            sector.firstwall = firstwall;
                            sector.nwalls = nwalls;
                            sector.zfloor = zfloor;
                            sector.zceil = zceil;
                            state.sectors.n += 1;
                        } else {
                            retval = -5;
                            break;
                        }
                    } else {
                        retval = -5;
                        break;
                    }
                }
                ScanState::ScanNone => {
                    retval = -6;
                    break;
                }
            }
        }
    }

    if retval != 0 {
        return Err(retval);
    }

    Ok(())
}

pub fn verline(x: i32, y0: i32, y1: i32, color: u32, state: &mut State) {
    for y in y0..=y1 {
        state.pixels[(y * (SCREEN_WIDTH as i32) + x) as usize] = color;
    }
}

pub fn point_in_sector(sector: &Sector, p: V2, state: &mut State) -> bool {
    for i in 0..sector.nwalls {
        let wall: &Wall = &state.walls.arr[sector.firstwall + i];

        if point_side(
            p.clone(),
            <V2i as Clone>::clone(&wall.a).v2i_to_v2(),
            <V2i as Clone>::clone(&wall.b).v2i_to_v2(),
        ) > 0.0
        {
            return false;
        }
    }

    true
}

pub fn render(state: &mut State) {
    for i in 0..SCREEN_WIDTH {
        state.y_hi[i] = (SCREEN_HEIGHT - 1) as u16;
        state.y_lo[i] = 0;
    }

    let mut sectdraw = [false; SECTOR_MAX as usize];

    // calculate edges of near/far planes (looking down +Y axis)
    let zdl = rotate(V2 { x: 0.0, y: 1.0 }, HFOV / 2.0);
    let zdr = rotate(V2 { x: 0.0, y: 1.0 }, -HFOV / 2.0);
    let znl = V2 {
        x: zdl.x * ZNEAR,
        y: zdl.y * ZNEAR,
    };
    let znr = V2 {
        x: zdr.x * ZNEAR,
        y: zdr.y * ZNEAR,
    };
    let zfl = V2 {
        x: zdl.x * ZFAR,
        y: zdl.y * ZFAR,
    };
    let zfr = V2 {
        x: zdr.x * ZFAR,
        y: zdr.y * ZFAR,
    };

    let mut queue = [QueueEntry {
        id: state.camera.sector as usize,
        x0: 0,
        x1: (SCREEN_WIDTH - 1) as i32,
    }];
    let mut queue_len = 1;

    #[derive(Clone, Copy)]
    struct QueueEntry {
        id: usize,
        x0: i32,
        x1: i32,
    }

    while queue_len != 0 {
        queue_len -= 1;
        let entry = queue[queue_len];

        if sectdraw[entry.id] {
            continue;
        }

        sectdraw[entry.id] = true;

        let sector = &state.sectors.arr[entry.id];

        for i in 0..sector.nwalls {
            let wall = &state.walls.arr[sector.firstwall + i];
            let op0 = world_pos_to_camera(wall.a.clone().v2i_to_v2(), state.clone().to_owned());
            let op1 = world_pos_to_camera(wall.b.clone().v2i_to_v2(), state.clone().to_owned());

            let mut cp0 = op0;
            let mut cp1 = op1;

            if cp0.y <= 0.0 && cp1.y <= 0.0 {
                continue;
            }

            let ap0 = normalize_angle(f32::atan2(cp0.y, cp0.x) - PI_2);
            let ap1 = normalize_angle(f32::atan2(cp1.y, cp1.x) - PI_2);

            if cp0.y < ZNEAR || cp1.y < ZNEAR || ap0 > HFOV / 2.0 || ap1 < -HFOV / 2.0 {
                let il = intersect_segs(&cp0, &cp1, &znl, &zfl);
                let ir = intersect_segs(&cp0, &cp1, &znr, &zfr);

                if !il.x.is_nan() {
                    cp0 = il;
                }

                if !ir.x.is_nan() {
                    cp1 = ir;
                }
            }

            if ap0 < ap1 {
                continue;
            }

            if (ap0 < -HFOV / 2.0 && ap1 < -HFOV / 2.0) || (ap0 > HFOV / 2.0 && ap1 > HFOV / 2.0) {
                continue;
            }

            let tx0 = screen_angle_to_x(ap0);
            let tx1 = screen_angle_to_x(ap1);

            if tx0 > entry.x1 || tx1 < entry.x0 {
                continue;
            }

            let wallshade = 16
                * (f32::sin(f32::atan2(
                    wall.b.clone().v2i_to_v2().x - wall.a.clone().v2i_to_v2().x,
                    wall.b.clone().v2i_to_v2().y - wall.a.clone().v2i_to_v2().y,
                )) + 1.0) as i32;
            let x0 = clamp(tx0, entry.x0, entry.x1);
            let x1 = clamp(tx1, entry.x0, entry.x1);
            let z_floor = sector.zfloor;
            let z_ceil = sector.zceil;
            let nz_floor = if wall.portal != 0 {
                state.sectors.arr[wall.portal].zfloor
            } else {
                0.0
            };
            let nz_ceil = if wall.portal != 0 {
                state.sectors.arr[wall.portal].zceil
            } else {
                0.0
            };
            let sy0 = ifnan((VFOV * SCREEN_HEIGHT as f32) / cp0.y, 1e10);
            let sy1 = ifnan((VFOV * SCREEN_HEIGHT as f32) / cp1.y, 1e10);
            let yf0 = (SCREEN_HEIGHT / 2) + ((z_floor - EYE_Z) * sy0) as i32;
            let yc0 = (SCREEN_HEIGHT / 2) + ((z_ceil - EYE_Z) * sy0) as i32;
            let yf1 = (SCREEN_HEIGHT / 2) + ((z_floor - EYE_Z) * sy1) as i32;
            let yc1 = (SCREEN_HEIGHT / 2) + ((z_ceil - EYE_Z) * sy1) as i32;
            let nyf0 = (SCREEN_HEIGHT / 2) + ((nz_floor - EYE_Z) * sy0) as i32;
            let nyc0 = (SCREEN_HEIGHT / 2) + ((nz_ceil - EYE_Z) * sy0) as i32;
            let nyf1 = (SCREEN_HEIGHT / 2) + ((nz_floor - EYE_Z) * sy1) as i32;
            let nyc1 = (SCREEN_HEIGHT / 2) + ((nz_ceil - EYE_Z) * sy1) as i32;
            let txd = tx1 - tx0;
            let yfd = yf1 - yf0;
            let ycd = yc1 - yc0;
            let nyfd = nyf1 - nyf0;
            let nycd = nyc1 - nyc0;

            for x in x0..=x1 {
                let shade = if x == x0 || x == x1 {
                    192
                } else {
                    255 - wallshade
                };

                let xp = ifnan(((x - tx0) / txd) as f32, 0.0);
                let tyf = (xp * yfd as f32) as i32 + yf0;
                let tyc = (xp * ycd as f32) as i32 + yc0;
                let yf = clamp(
                    tyf,
                    state.y_lo[x as usize].into(),
                    state.y_hi[x as usize].into(),
                );
                let yc = clamp(
                    tyc,
                    state.y_lo[x as usize].into(),
                    state.y_hi[x as usize].into(),
                );

                if yf > state.y_lo[x as usize].into() {
                    verline(
                        x,
                        state.y_lo[x as usize].into(),
                        yf,
                        0xFFFF0000,
                        &mut state.clone().to_owned(),
                    );
                }

                if yc < state.y_hi[x as usize] as i32 {
                    verline(
                        x,
                        yc,
                        state.y_hi[x as usize].into(),
                        0xFF00FFFF,
                        &mut state.clone().to_owned(),
                    );
                }

                if wall.portal != 0 {
                    let tnyf = (xp * nyfd as f32) as i32 + nyf0;
                    let tnyc = (xp * nycd as f32) as i32 + nyc0;
                    let nyf = clamp(
                        tnyf,
                        state.y_lo[x as usize].into(),
                        state.y_hi[x as usize].into(),
                    );
                    let nyc = clamp(
                        tnyc,
                        state.y_lo[x as usize].into(),
                        state.y_hi[x as usize].into(),
                    );

                    verline(
                        x,
                        nyc,
                        yc,
                        abgr_mul(0xFF00FF00, shade as u32),
                        &mut state.clone(),
                    ); // Black Magic
                    verline(
                        x,
                        yf,
                        nyf,
                        abgr_mul(0xFF0000FF, shade as u32),
                        &mut state.clone(),
                    ); // No touch

                    state.y_hi[x as usize] = clamp(
                        min(
                            min(yc.try_into().unwrap(), nyc.try_into().unwrap()),
                            state.y_hi[x as usize],
                        ),
                        0,
                        (SCREEN_HEIGHT - 1) as u16,
                    );
                    state.y_lo[x as usize] = clamp(
                        max(
                            max(yf.try_into().unwrap(), nyf.try_into().unwrap()),
                            state.y_lo[x as usize],
                        ),
                        0,
                        (SCREEN_HEIGHT - 1) as u16,
                    );
                } else {
                    verline(
                        x,
                        yf,
                        yc,
                        abgr_mul(0xFFD0D0D0, shade as u32),
                        &mut state.clone(),
                    );
                }

                if state.sleepy {
                    present(&mut state.clone());
                    let ten_millis = std::time::Duration::from_millis(10);

                    std::thread::sleep(ten_millis)
                }
            }

            if wall.portal != 0 {
                assert!(queue_len != QUEUE_MAX as usize, "out of queue space");
                queue[queue_len] = QueueEntry {
                    id: wall.portal,
                    x0,
                    x1,
                };
                queue_len += 1;
            }
        }
    }

    state.sleepy = false;
}

pub fn present(state: &mut State) {
    let mut px: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut pitch: i32 = 0;

    unsafe {
        SDL_LockTexture(state.texture, std::ptr::null(), &mut px, &mut pitch);
        let px_slice = slice::from_raw_parts_mut(px as *mut u8, (SCREEN_HEIGHT * pitch) as usize);
        let pixels_slice = slice::from_raw_parts(
            state.pixels.as_ptr() as *const u8,
            SCREEN_WIDTH * SCREEN_HEIGHT as usize * 4,
        );
        px_slice.copy_from_slice(pixels_slice);
        SDL_UnlockTexture(state.texture);

        SDL_SetRenderTarget(state.renderer, std::ptr::null_mut());
        SDL_SetRenderDrawColor(state.renderer, 0, 0, 0, 0xFF);
        SDL_SetRenderDrawBlendMode(state.renderer, sdl2_sys::SDL_BlendMode::SDL_BLENDMODE_NONE);

        SDL_RenderClear(state.renderer);
        SDL_RenderCopyEx(
            state.renderer,
            state.texture,
            std::ptr::null(),
            std::ptr::null(),
            0.0,
            std::ptr::null(),
            sdl2_sys::SDL_RendererFlip::SDL_FLIP_VERTICAL,
        );

        SDL_SetTextureBlendMode(state.debug, sdl2_sys::SDL_BlendMode::SDL_BLENDMODE_BLEND);
        SDL_RenderCopy(
            state.renderer,
            state.debug,
            std::ptr::null(),
            &SDL_Rect {
                x: 0,
                y: 0,
                w: 512,
                h: 512,
            },
        );
        SDL_RenderPresent(state.renderer);
    }
}
