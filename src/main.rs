extern crate sdl2_sys;

pub mod internaltypes;
pub mod res;

use crate::res::game::render;
use crate::res::game::State;
use crate::res::util::constants::*;
use crate::res::util::kinds::*;

use sdl2_sys::*;

fn main() {
    lazy_static! {
    static ref state: State = State {
        
    }
    };
}

    println!("Hello World!");
    // game -> things -> attributes
    let x = c();
}

fn c() {
    unsafe {
        let init_res = SDL_Init(SDL_INIT_VIDEO);
        assert!(init_res == 0, "SDL failed to initialize: ");
        
    }
}
