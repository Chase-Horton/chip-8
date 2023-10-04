#![allow(unused)]

use std::fs;

static FONT_START_ADDRESS: u16 = 0x00;
fn load_fonts() -> [u8; 80] {
    [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ]
}

struct EmulatedRam {
    data: [u8; 0x1000 as usize], // 4096 bytes of memory
}
impl EmulatedRam {
    fn new() -> Self {
        let mut ram = EmulatedRam { data: [0; 0x1000] };
        let fonts = load_fonts();
        let mut address = FONT_START_ADDRESS;
        for font in fonts.iter() {
            // println!("{:#02x}", font);
            ram.write_byte(address, *font);
            address += 1
        }
        ram
    }

    fn read_byte(&self, address: u16) -> u8 {
        if address > 0xFFF {
            panic!("Ram: tried to access memory out of bounds")
        }
        self.data[address as usize]
    }
    fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    fn load_program(&mut self, data: Vec<u8>) {
        let mut counter = 0x200;
        for byte in data {
            self.write_byte(counter, byte);
            counter += 1;
        }
    }

    fn load_program_from_file(&mut self, file_path: &str) {
        let contents = fs::read(file_path).expect("failed to open program from file");
        self.load_program(contents);
    }
}

struct EmulatedScreen {
    pixels: [[bool; 64]; 32],
}
impl EmulatedScreen {
    fn new() -> Self {
        EmulatedScreen {
            pixels: [[false; 64]; 32],
        }
    }
    fn put_pixel(&mut self, x: u8, y: u8, pix: bool) -> Result<bool, &str> {
        // need to wrap around take abs (y) % 32 and abs(x) % 64
        if x < 64 && y < 32 {
            self.pixels[y as usize][x as usize] = pix;
            Ok(pix)
        } else {
            Err("Attempting to put pixel out of bounds")
        }
    }
    fn clear(&mut self) {
        for x in 0..64 {
            for y in 0..32 {
                self.pixels[y][x] = false;
            }
        }
    }
}

struct DelayTimer {
    val: u8,
}
impl DelayTimer {
    fn new() -> DelayTimer {
        DelayTimer { val: 0 }
    }
}

#[derive(Debug, PartialEq)]
enum OpCode {
    CLR,              //clear screen
    JMP(u16),         //1NNN jmp to NNN
    SET(u8, u8),      //6XNN set register VX , X is addr in v_registers of 0-F
    ADD(u8, u8),      //7XNN add value to register VX
    SetAddrReg(u16),  //ANNN set index register I
    DXYN(u8, u8, u8), //display/draw
    UNFINISHED,
}

struct Chip8 {
    pc: u16,
    i_reg: u16,
    address_stack: Vec<u16>,
    stack_pointer: u8,
    delay_timer: DelayTimer,
    v_registers: [u8; 16],
    screen: EmulatedScreen,
    ram: EmulatedRam,
}
impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            pc: 0,
            i_reg: 0,
            address_stack: Vec::new(),
            stack_pointer: 0,
            delay_timer: DelayTimer::new(),
            v_registers: [0x0; 16],
            screen: EmulatedScreen::new(),
            ram: EmulatedRam::new(),
        }
    }

    fn fetch(&mut self) -> u16 {
        let byte: u16 =
            (self.ram.read_byte(self.pc) as u16) << 8 | self.ram.read_byte(self.pc + 1) as u16;
        self.pc += 2;
        byte
    }

    fn decode(&mut self, instruction: u16) -> OpCode {
        let upper_byte = ((instruction & 0xFF00) >> 8) as u8;
        let lower_byte = (instruction & 0x00FF) as u8;
        let op = (upper_byte & 0xF0) >> 4;
        let x = upper_byte & 0x0F;
        let y = (lower_byte & 0xF0) >> 4;
        let d = lower_byte & 0x0F;
        let nnn = instruction & 0x0FFF;
        // println!("upper byte {:2x}", upper_byte);
        // println!("lower byte {:2x}", lower_byte);
        // println!("op {:2x}", op);
        // println!("x nibble {:1x}", x);
        // println!("y nibble {:1x}", y);
        // println!("nnn {:3x}", nnn);
        match (op, x, y, d) {
            (0, 0, 0xE, 0) => OpCode::CLR,
            (1, _, _, _) => OpCode::JMP(nnn),
            (6, _, _, _) => OpCode::SET(x, lower_byte),
            (7, _, _, _) => OpCode::ADD(x, lower_byte),
            (0xA, _, _, _) => OpCode::SetAddrReg(nnn),
            (0xD, _, _, _) => OpCode::DXYN(x, y, d),
            (_, _, _, _) => OpCode::UNFINISHED,
        }
    }
    fn execute(&mut self, op_code: OpCode) {
        match op_code {
            OpCode::CLR => self.screen.clear(),
            OpCode::JMP(addr) => self.pc = addr,
            OpCode::ADD(v_x, kk) => self.v_registers[v_x as usize] += kk,
            OpCode::SET(v_x, kk) => self.v_registers[v_x as usize] = kk,
            OpCode::SetAddrReg(addr) => self.i_reg = addr,
            OpCode::DXYN(x, y, n) => {
                //Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                //The interpreter reads n bytes from memory, starting at the address stored in I.
                //These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
                //Sprites are XORed onto the existing screen. If this causes any pixels to be erased,
                //VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the
                //coordinates of the display, it wraps around to the opposite side of the screen.
                //See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
                let v_x = self.v_registers[x as usize];
                let v_y = self.v_registers[y as usize];
                // set flag to 0
                self.v_registers[0xF] = 0x0;
                for row in 0..n {
                    let spirte_byte_from_mem = self.ram.read_byte(self.i_reg + row as u16);
                    for col in 0..8 {
                        // shift to right and get last bit
                        let pixel = (spirte_byte_from_mem >> (7 - col)) & 1;
                        // wrap around
                        let screen_x = ((v_x + col) % 64) as usize;
                        let screen_y = ((v_y + row) % 32) as usize;
                        // set flag
                        if pixel == 1 {
                            if self.screen.pixels[screen_y][screen_x] {
                                self.v_registers[0xF] = 0x1;
                            }
                            self.screen.pixels[screen_y][screen_x] ^= true;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    fn cycle(&mut self) {
        let instruction = self.fetch();
        println!("{:04x}", instruction);
        let op_code = self.decode(instruction);
        println!("{:?}", op_code)
        // self.execute(op_code)
    }
    fn run(&mut self) {
        self.pc = 0x200;
        for _ in 0..50 {
            self.cycle();
        }
    }
}
fn main() {
    let mut chip8 = Chip8::new();
    chip8.ram.load_program_from_file("1-chip8-logo.ch8");
    chip8.run();
    // println!("{:?}", ram.data)
    //let op_code = chip8.decode(0x00E0);
    //println!("{:?}", op_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Ram Tests
    #[test]
    #[should_panic]
    fn ram_out_of_bounds() {
        let ram = EmulatedRam::new();
        ram.read_byte(0x1000);
    }
    #[test]
    fn ram_read() {
        let ram = EmulatedRam::new();
        for i in FONT_START_ADDRESS + 80..=0xFFF {
            assert_eq!(ram.read_byte(i), 0);
        }
    }
    #[test]
    fn ram_write_read() {
        let mut ram = EmulatedRam::new();
        for i in 0..=0xFFF {
            ram.write_byte(i, 1);
            assert_eq!(ram.read_byte(i), 1);
        }
    }
    #[test]
    fn ram_font_load() {
        let ram = EmulatedRam::new();
        let fonts: [u8; 80] = load_fonts();
        let mut fonts_from_ram = [0; 80];
        let mut c = 0;
        for address in FONT_START_ADDRESS..(FONT_START_ADDRESS + 80) {
            fonts_from_ram[c] = ram.read_byte(address);
            c += 1;
        }
        assert_eq!(ram.read_byte(FONT_START_ADDRESS), 0xF0);
        assert_eq!(ram.read_byte(FONT_START_ADDRESS + 1), 0x90);
        assert_eq!(ram.read_byte(FONT_START_ADDRESS + 79), 0x80);
        assert_eq!(fonts_from_ram, fonts);
    }
    // Screen Tests
    #[test]
    fn screen_write_every_pixel() {
        let mut screen = EmulatedScreen::new();
        for y in 0..32 {
            for x in 0..64 {
                let res = screen.put_pixel(x, y, true);
                assert_eq!(res, Ok(true));
            }
        }
    }
    #[test]
    fn screen_write_oob() {
        let mut screen = EmulatedScreen::new();
        let bad_res = screen.put_pixel(255, 4, true);
        assert_eq!(bad_res, Err("Attempting to put pixel out of bounds"));
        let bad_res = screen.put_pixel(64, 4, true);
        assert_eq!(bad_res, Err("Attempting to put pixel out of bounds"));
        let bad_res = screen.put_pixel(30, 36, true);
        assert_eq!(bad_res, Err("Attempting to put pixel out of bounds"));
        let bad_res = screen.put_pixel(100, 100, true);
        assert_eq!(bad_res, Err("Attempting to put pixel out of bounds"));
        let bad_res = screen.put_pixel(63, 4, true);
        assert_ne!(bad_res, Err("Attempting to put pixel out of bounds"));
    }

    #[test]
    fn instruction_fetch() {
        //should fetch instruction and increment pc by 2
        let mut chip8 = Chip8::new();
        chip8.pc = 0;
        chip8.ram.write_byte(0, 0xAB);
        chip8.ram.write_byte(1, 0xCD);
        chip8.ram.write_byte(2, 0x00);
        chip8.ram.write_byte(3, 0xE0);
        let res = chip8.fetch();
        assert_eq!(res, 0xABCD);
        //check that it increments
        let res = chip8.fetch();
        assert_eq!(res, 0x00E0);
    }
    // decoder
    #[test]
    fn decoder_test() {
        let mut chip8 = Chip8::new();

        let res = chip8.decode(0x00E0);
        assert_eq!(res, OpCode::CLR);

        let res = chip8.decode(0x1ABC);
        assert_eq!(res, OpCode::JMP(0xABC));

        let res = chip8.decode(0x7B44);
        assert_eq!(res, OpCode::ADD(0xB, 0x44));

        let res = chip8.decode(0x6B44);
        assert_eq!(res, OpCode::SET(0xB, 0x44));

        let res = chip8.decode(0xACDE);
        assert_eq!(res, OpCode::SetAddrReg(0x0CDE));

        let res = chip8.decode(0xD123);
        assert_eq!(res, OpCode::DXYN(1, 2, 3))
    }
    // execute
    #[test]
    fn execute_clr() {
        let mut chip8 = Chip8::new();
        chip8.screen.put_pixel(0, 0, true);
        assert_eq!(chip8.screen.pixels[0][0], true);
        chip8.screen.put_pixel(20, 20, true);
        assert_eq!(chip8.screen.pixels[20][20], true);
        let res = chip8.decode(0x00E0);
        chip8.execute(res);
        assert_eq!(chip8.screen.pixels[0][0], false);
        assert_eq!(chip8.screen.pixels[20][20], false);
    }
    #[test]
    fn execute_jmp() {
        let mut chip8 = Chip8::new();
        let res = chip8.decode(0x1ABC);
        chip8.execute(res);
        assert_eq!(chip8.pc, 0xABC)
    }
    #[test]
    fn execute_add() {
        let mut chip8 = Chip8::new();
        assert_eq!(chip8.v_registers[0xC], 0);
        let res = chip8.decode(0x7C04);
        chip8.execute(res);
        assert_eq!(chip8.v_registers[0xC], 0x04);
        let res = chip8.decode(0x7C04);
        chip8.execute(res);
        assert_eq!(chip8.v_registers[0xC], 0x08);
        let res = chip8.decode(0x7C04);
        chip8.execute(res);
        assert_eq!(chip8.v_registers[0xC], 0x0C);
    }
    #[test]
    fn execute_set() {
        let mut chip8 = Chip8::new();
        let res = chip8.decode(0x6C44);
        chip8.execute(res);
        assert_eq!(chip8.v_registers[0xC], 0x44);
    }
    #[test]
    fn execute_set_addr_reg() {
        let mut chip8 = Chip8::new();
        let res = chip8.decode(0xAC44);
        chip8.execute(res);
        assert_eq!(chip8.i_reg, 0xC44);
    }
    #[test]
    fn execute_display() {
        let mut chip8 = Chip8::new();
        //test basic display of 1 byte
        chip8.ram.write_byte(0x300, 0b11011001);
        chip8.i_reg = 0x300;
        chip8.v_registers[0] = 0;
        chip8.v_registers[1] = 0;
        let res = chip8.decode(0xD011);
        chip8.execute(res);
        assert_eq!(chip8.screen.pixels[0][0], true);
        assert_eq!(chip8.screen.pixels[0][1], true);
        assert_eq!(chip8.screen.pixels[0][2], false);
        assert_eq!(chip8.screen.pixels[0][3], true);
        assert_eq!(chip8.screen.pixels[0][4], true);
        assert_eq!(chip8.screen.pixels[0][5], false);
        assert_eq!(chip8.screen.pixels[0][6], false);
        assert_eq!(chip8.screen.pixels[0][7], true);
    }
    #[test]
    fn execute_display_two_rows() {
        let mut chip8 = Chip8::new();
        //test basic display of 1 byte
        chip8.ram.write_byte(0x300, 0b11011001);
        chip8.ram.write_byte(0x301, 0b10101010);
        chip8.i_reg = 0x300;
        chip8.v_registers[0] = 0;
        chip8.v_registers[1] = 0;
        let res = chip8.decode(0xD012);
        chip8.execute(res);
        assert_eq!(chip8.screen.pixels[0][0], true);
        assert_eq!(chip8.screen.pixels[0][1], true);
        assert_eq!(chip8.screen.pixels[0][2], false);
        assert_eq!(chip8.screen.pixels[0][3], true);
        assert_eq!(chip8.screen.pixels[0][4], true);
        assert_eq!(chip8.screen.pixels[0][5], false);
        assert_eq!(chip8.screen.pixels[0][6], false);
        assert_eq!(chip8.screen.pixels[0][7], true);
        assert_eq!(chip8.screen.pixels[1][0], true);
        assert_eq!(chip8.screen.pixels[1][1], false);
        assert_eq!(chip8.screen.pixels[1][2], true);
        assert_eq!(chip8.screen.pixels[1][3], false);
        assert_eq!(chip8.screen.pixels[1][4], true);
        assert_eq!(chip8.screen.pixels[1][5], false);
        assert_eq!(chip8.screen.pixels[1][6], true);
        assert_eq!(chip8.screen.pixels[1][7], false);
    }
}
