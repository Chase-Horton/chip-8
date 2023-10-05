#![allow(unused)]
mod cpu;
use cpu::Chip8;
fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_program("1-chip8-logo.ch8");
    chip8.pc = 0x200;
    chip8.cycle();
    // println!("{:?}", chip8.ram.data)
    //let op_code = chip8.decode(0x00E0);
    //println!("{:?}", op_code)
}
