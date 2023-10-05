#![allow(unused)]
mod cpu;
use cpu::Chip8;
// fn main() {
//     let mut chip8 = Chip8::new();
//     chip8.load_program("1-chip8-logo.ch8");
//     chip8.pc = 0x200;
//     chip8.cycle();
//     // println!("{:?}", chip8.ram.data)
//     //let op_code = chip8.decode(0x00E0);
//     //println!("{:?}", op_code)
// }
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::env;
const WIDTH: usize = 1280;
const HEIGHT: usize = 640;
const BACKGROUND_COLOR: u32 = 0x0;
const ACTIVE_COLOR: u32 = 0xFFFFFFFF;
const SCALE: usize = 20;
fn main() {
    let mut fake_screen: [[bool; 64]; 32] = [[false; 64]; 32];
    for y in 0..32 {
        for x in 0..64 {
            let pix = y % 2 == 0;
            fake_screen[y][x] = pix;
        }
    }
    let args: Vec<String> = env::args().collect();
    let mut chip8 = Chip8::new();
    chip8.load_program(&args[1]);
    chip8.pc = 0x200;
    let mut fake_screen = chip8.get_screen();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new("Chip8 Interpreter", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::Space, KeyRepeat::Yes) {
            chip8.cycle();
            fake_screen = chip8.get_screen();
        }
        for x in 0..64 {
            for y in 0..32 {
                if fake_screen[y][x] {
                    for i in x * SCALE..x * SCALE + SCALE {
                        for j in y * SCALE..y * SCALE + SCALE {
                            let index = j * WIDTH + i;
                            buffer[index] = ACTIVE_COLOR;
                        }
                    }
                } else {
                    for i in x * SCALE..x * SCALE + SCALE {
                        for j in y * SCALE..y * SCALE + SCALE {
                            let index = j * WIDTH + i;
                            buffer[index] = BACKGROUND_COLOR;
                        }
                    }
                }
            }
        }
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
