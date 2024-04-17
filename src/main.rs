extern crate sdl2;


pub mod internaltypes;
pub mod res;
use crate::internaltypes::thing::Thing;

fn main() {
    println!("Hello World!");
    // game -> things -> attributes
    let x: Thing = Thing::new("My first Thing".to_string());

    println!("{:?}", x);
}