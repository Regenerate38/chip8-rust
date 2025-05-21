#![allow(non_snake_case)]

use raylib::prelude::*;
use std::{env, fs};

mod chip8;
use chip8::{Chip8, *};

const TICKS_PER_FRAME: usize = 10;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{} Enter Path to ROM", &args[0]);
        std::process::exit(0)
    }

    std::process::exit(match run(&args[1]) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error {:?}", err);
            1
        }
    });
}

fn run(rom_path: &str) -> Result<(), String> {
    println!("Initializing Raylib");
    let window_title = format!("Chip 8: {rom_path}");
    // Initializing Raylib and Chip8
    let mut mychip8 = Chip8::new();
    let (mut rl, thread) = raylib::init().size(640, 640).title(&window_title).build();

    println!("Loading ROM {rom_path}");
    let rom = fs::read(rom_path).map_err(|e| format!("Cannot read ROM: {}", e))?;
    let count = mychip8.loadProgram(&rom)?;
    println!("Loaded the program: {:?} bytes", count);

    rl.set_target_fps(60);
    while !rl.window_should_close() {
        let key = rl.get_key_pressed();
        if key == Some(KeyboardKey::KEY_ESCAPE) {
            break;
        }

        let keys_to_check = [
            (KeyboardKey::KEY_ONE, 1),
            (KeyboardKey::KEY_TWO, 2),
            (KeyboardKey::KEY_THREE, 3),
            (KeyboardKey::KEY_FOUR, 12),
            (KeyboardKey::KEY_Q, 4),
            (KeyboardKey::KEY_W, 5),
            (KeyboardKey::KEY_E, 6),
            (KeyboardKey::KEY_R, 13),
            (KeyboardKey::KEY_A, 7),
            (KeyboardKey::KEY_S, 8),
            (KeyboardKey::KEY_D, 9),
            (KeyboardKey::KEY_F, 14),
            (KeyboardKey::KEY_Z, 10),
            (KeyboardKey::KEY_X, 0),
            (KeyboardKey::KEY_C, 11),
            (KeyboardKey::KEY_V, 15),
        ];

        for (key, value) in keys_to_check {
            if rl.is_key_pressed(key) {
                mychip8.keypress(value, true);
            }
            if rl.is_key_released(key) {
                mychip8.keypress(value, false);
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            mychip8.executeCycle();
        }
        let mut d = rl.begin_drawing(&thread);
        draw_frame(&mychip8, &mut d);
    }
    Ok(())
}

fn draw_frame(chip8: &chip8::Chip8, d: &mut RaylibDrawHandle) {
    d.clear_background(Color::BLACK);
    let scale = 10;

    for y in 0..chip8::SCREEN_HEIGHT {
        for x in 0..chip8::SCREEN_WIDTH {
            if chip8.is_pixel_set(x, y) {
                d.draw_rectangle(
                    x as i32 * scale,
                    y as i32 * scale,
                    scale,
                    scale,
                    Color::WHITE,
                );
            }
        }
    }
}
