#![allow(non_snake_case)]

use raylib::prelude::*;

mod chip8;
use chip8::{Chip8, *};

fn main() {
    let mychip8 = Chip8::new();

    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}
