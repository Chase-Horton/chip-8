mod opcodes;
use opcodes::OpCode;
mod ram;
use ram::EmulatedRam;
mod screen;
use screen::EmulatedScreen;
use std::{fs, ops::Add};
use rand;

struct DelayTimer {
    val: u8,
}
impl DelayTimer {
    fn new() -> DelayTimer {
        DelayTimer { val: 0 }
    }
}

pub struct Chip8 {
    pub pc: u16,
    pub i_reg: u16,
    address_stack: Vec<u16>,
    stack_pointer: u8,
    delay_timer: DelayTimer,
    pub v_registers: [u8; 16],
    screen: EmulatedScreen,
    pub ram: EmulatedRam,
}
impl Chip8 {
    pub fn new() -> Chip8 {
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
    pub fn get_screen(&self) -> [[bool; 64]; 32] {
        self.screen.get_screen()
    }
    pub fn load_program(&mut self, path: &str) {
        self.ram.load_program_from_file(path);
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
            (0, 0, 0xE, 0xE) => OpCode::RET,
            (0x1, _, _, _) => OpCode::JMP(nnn),
            (0x2, _, _, _) => OpCode::CALL(nnn),
            (0x3, _, _, _) => OpCode::SkipEqualNN(x, lower_byte),
            (0x4, _, _, _) => OpCode::SkipNotEqualNN(x, lower_byte),
            (0x5, _, _, _) => OpCode::SkipEqualXY(x, y),
            (0x6, _, _, _) => OpCode::SET(x, lower_byte),
            (0x7, _, _, _) => OpCode::ADD(x, lower_byte),
            (0x8, _, _, 0) => OpCode::LDXY(x, y),
            (0x8, _, _, 1) => OpCode::BOR(x, y),
            (0x8, _, _, 2) => OpCode::BAND(x, y),
            (0x8, _, _, 3) => OpCode::BXOR(x, y),
            (0x8, _, _, 4) => OpCode::AddXY(x, y),
            (0x8, _, _, 5) => OpCode::SubXY(x, y),
            (0x8, _, _, 6) => OpCode::SHR(x, y),
            (0x8, _, _, 7) => OpCode::SUBN(x, y),
            (0x8, _, _, 0xE) => OpCode::SHL(x, y),
            (0x9, _, _, 0) => OpCode::SkipNotEqualXY(x, y),
            (0xA, _, _, _) => OpCode::SetAddrReg(nnn),
            (0xB, _, _, _) => OpCode::JumpPlusV0(nnn),
            (0xC, _, _, _) => OpCode::RAND(x, lower_byte),
            (0xD, _, _, _) => OpCode::DXYN(x, y, d),
            // (0xE, _, 9, 0xE) => OpCode::SkipKeyPressed(x),
            // (0xE, _, 0xA, 1) => OpCode::SkipKeyNotPressed(x),
            // (0xF, _, 0, 7) => OpCode::SetVxToDelayTimer(x),
            // (0xF, _, 0, 0xA) => OpCode::WaitForKeyPress(x),
            // (0xF, _, 1, 5) => OpCode::SetDelayTimer(x),
            // (0xF, _, 1, 8) => OpCode::SetSoundTimer(x),
            (0xF, _, 1, 0xE) => OpCode::AddVxToI(x),
            // (0xF, _, 2, 9) => OpCode::SetIToSprite(x),
            (0xF, _, 3, 3) => OpCode::SaveBCD(x),
            (0xF, _, 5, 5) => OpCode::StoreV0ToVx(x),
            (0xF, _, 6, 5) => OpCode::ReadV0ToVx(x),
            (_, _, _, _) => OpCode::UNFINISHED,
        }
    }
    fn execute(&mut self, op_code: OpCode) {
        match op_code {
            OpCode::CLR => self.screen.clear(),
            OpCode::JMP(addr) => self.pc = addr,
            OpCode::ADD(v_x, kk) => self.v_registers[v_x as usize] = self.v_registers[v_x as usize].wrapping_add(kk),
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
                    self.v_registers[0xF] =
                        self.screen.write_byte(v_x, v_y + row, spirte_byte_from_mem)
                }
            }
            OpCode::CALL(nnn) => {
                self.stack_pointer += 1;
                self.address_stack.push(self.pc);
                self.pc = nnn;
            }
            OpCode::SkipEqualNN(x, kk) => {
                if self.v_registers[x as usize] == kk {
                    self.pc += 2;
                }
            }
            OpCode::SkipNotEqualNN(x, kk) => {
                if self.v_registers[x as usize] != kk {
                    self.pc += 2;
                }
            }
            OpCode::SkipEqualXY(x, y) => {
                if self.v_registers[x as usize] == self.v_registers[y as usize] {
                    self.pc += 2;
                }
            }
            OpCode::LDXY(x, y) => {
                self.v_registers[x as usize] = self.v_registers[y as usize];
            }
            OpCode::BOR(x, y) => {
                self.v_registers[x as usize] |= self.v_registers[y as usize];
            }
            OpCode::BAND(x, y) => {
                self.v_registers[x as usize] &= self.v_registers[y as usize];
            }
            OpCode::BXOR(x, y) => {
                self.v_registers[x as usize] ^= self.v_registers[y as usize];
            }
            OpCode::AddXY(x, y) => {
                let res = self.v_registers[x as usize] as u16 + self.v_registers[y as usize] as u16;
                if res > 255 {
                    self.v_registers[x as usize] = res as u8;
                    self.v_registers[0xF] = 1;
                }
                else {
                    self.v_registers[x as usize] += self.v_registers[y as usize];
                    self.v_registers[0xF] = 0;
                }
            }
            OpCode::SubXY(x, y) => {
                let x = x as usize;
                let y = y as usize;
                let borrow = if self.v_registers[x] >= self.v_registers[y] { 1 } else { 0 };
                self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
                self.v_registers[0x0f] = borrow;
            }
            OpCode::SHR(x, y) => {
                let bit = self.v_registers[y as usize] & 0x1;
                self.v_registers[x as usize] = self.v_registers[y as usize] >> 1;
                self.v_registers[0xF] = bit;
            }
            OpCode::SUBN(x, y) => {
                let x = x as usize;
                let y = y as usize;
                let borrow = if self.v_registers[y] >= self.v_registers[x] { 1 } else { 0 };
                self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);
                self.v_registers[0x0f] = borrow;
            }
            OpCode::SHL(x, y) => {
                let bit = self.v_registers[y as usize] >> 7 & 0x1;; 
                self.v_registers[x as usize] = self.v_registers[y as usize] << 1;
                self.v_registers[0xF] = bit;
            }
            OpCode::SkipNotEqualXY(x, y) => {
                if self.v_registers[x as usize] != self.v_registers[y as usize] {
                    self.pc += 2;
                }
            }
            OpCode::JumpPlusV0(nnn) => {
                self.pc = nnn + self.v_registers[0] as u16;
            }
            OpCode::RAND(x, kk) => {
                let rand = rand::random::<u8>();
                self.v_registers[x as usize] = rand & kk;
            },
            OpCode::RET => {
                self.stack_pointer -= 1;
                self.pc = self.address_stack.pop().unwrap();
            },
            OpCode::AddVxToI(x) => {
                self.i_reg += self.v_registers[x as usize] as u16;
            },
            OpCode::SaveBCD(x) => {
                let val = self.v_registers[x as usize];
                self.ram.write_byte(self.i_reg, val / 100);
                self.ram.write_byte(self.i_reg + 1, (val / 10) % 10);
                self.ram.write_byte(self.i_reg + 2, val % 10);
            },
            OpCode::StoreV0ToVx(x) => {
                for i in 0..=x {
                    self.ram.write_byte(self.i_reg + i as u16, self.v_registers[i as usize]);
                }
            },
            OpCode::ReadV0ToVx(x) => {
                for i in 0..=x {
                    self.v_registers[i as usize] = self.ram.read_byte(self.i_reg + i as u16);
                }
            },
            
            OpCode::UNFINISHED => {}
        }
    }
    // function to print out the data in the registers and the i register as well as the stack and stack pointer
    pub fn debug_print_data(&mut self){
        println!("pc: {:x}", self.pc);
        println!("i register: {:x}", self.i_reg);
        println!("stack pointer: {:x}", self.stack_pointer);
        println!("stack: {:?}", self.address_stack);
        println!("v registers: {:?}", self.v_registers);
    }
    pub fn cycle(&mut self) {
        let instruction = self.fetch();
        println!("{:04x}", instruction);
        let op_code = self.decode(instruction);
        println!("{:?}", op_code);
        self.execute(op_code)
    }
    fn run(&mut self) {
        self.pc = 0x200;
        for _ in 0..50 {
            self.cycle();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    //execute
    #[test]
    fn execute_clr() {
        let mut chip8 = Chip8::new();
        chip8.screen.put_pixel(0, 0, true);
        assert_eq!(chip8.screen.get_pixel(0, 0), true);
        chip8.screen.put_pixel(20, 20, true);
        assert_eq!(chip8.screen.get_pixel(20, 20), true);
        let res = chip8.decode(0x00E0);
        chip8.execute(res);
        assert_eq!(chip8.screen.get_pixel(0, 0), false);
        assert_eq!(chip8.screen.get_pixel(20, 20), false);
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
    fn execute_call() {
        let mut chip8 = Chip8::new();
        let res = chip8.decode(0x2111);
        chip8.pc = 0x50;
        chip8.execute(res);
        assert_eq!(chip8.stack_pointer, 0x1);
        assert_eq!(chip8.address_stack[0], 0x50);
        assert_eq!(chip8.pc, 0x111)
    }
    #[test]
    fn execute_se() {
        let mut chip8 = Chip8::new();
        chip8.pc = 0x00;
        chip8.v_registers[0] = 0x11;
        let res = chip8.decode(0x3011);
        chip8.execute(res);
        assert_eq!(chip8.pc, 2);
        let res = chip8.decode(0x3014);
        chip8.execute(res);
        assert_eq!(chip8.pc, 2);
    }
    #[test]
    fn execute_sne() {
        let mut chip8 = Chip8::new();
        chip8.pc = 0x00;
        chip8.v_registers[0] = 0x11;
        let res = chip8.decode(0x4015);
        chip8.execute(res);
        assert_eq!(chip8.pc, 2);
        let res = chip8.decode(0x4011);
        chip8.execute(res);
        assert_eq!(chip8.pc, 2);
    }
    #[test]
    fn execute_se_xy() {
        let mut chip8 = Chip8::new();
        chip8.pc = 0x00;
        chip8.v_registers[0] = 0x11;
        chip8.v_registers[1] = 0x11;
        let res = chip8.decode(0x5010);
        chip8.execute(res);
        assert_eq!(chip8.pc, 2);
        chip8.v_registers[0] = 0x14;
        chip8.v_registers[1] = 0x11;
        let res = chip8.decode(0x5010);
        chip8.execute(res);
        assert_eq!(chip8.pc, 2);
    }
    #[test]
    fn execute_ldxy() {
        let mut cpu = Chip8::new();
        cpu.v_registers[1] = 10;
        cpu.execute(OpCode::LDXY(0, 1));
        assert_eq!(cpu.v_registers[0], 10);
    }
    #[test]
    fn execute_bor() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0b1010;
        cpu.v_registers[1] = 0b0101;
        cpu.execute(OpCode::BOR(0, 1));
        assert_eq!(cpu.v_registers[0], 0b1111);
    }
    #[test]
    fn execute_band() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0b1010;
        cpu.v_registers[1] = 0b0101;
        cpu.execute(OpCode::BAND(0, 1));
        assert_eq!(cpu.v_registers[0], 0b0000);
    }
    #[test]
    fn execute_bxor() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0b1010;
        cpu.v_registers[1] = 0b0101;
        cpu.execute(OpCode::BXOR(0, 1));
        assert_eq!(cpu.v_registers[0], 0b1111);
        cpu.v_registers[0] = 0b1111;
        cpu.v_registers[1] = 0b0101;
        cpu.execute(OpCode::BXOR(0, 1));
        assert_eq!(cpu.v_registers[0], 0b1010);
    }
    #[test]
    fn execute_addxy() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0x10;
        cpu.v_registers[1] = 0x10;
        cpu.execute(OpCode::AddXY(0, 1));
        assert_eq!(cpu.v_registers[0], 0x20);
        assert_eq!(cpu.v_registers[0xF], 0x0);
        cpu.v_registers[0] = 0xFF;
        cpu.v_registers[1] = 0x02;
        cpu.execute(OpCode::AddXY(0, 1));
        assert_eq!(cpu.v_registers[0], 0x01);
        assert_eq!(cpu.v_registers[0xF], 0x1);
    }
    #[test]
    fn execute_subxy() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0b11;
        cpu.v_registers[1] = 0b10;
        cpu.execute(OpCode::SubXY(0, 1));
        assert_eq!(cpu.v_registers[0], 0b01);
        assert_eq!(cpu.v_registers[0xF], 0x1);
        cpu.v_registers[0] = 0b01;
        cpu.v_registers[1] = 0b10;
        cpu.execute(OpCode::SubXY(0, 1));
        assert_eq!(cpu.v_registers[0], 255);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }
    //TODO: update tests for new implementation
/*     #[test]
    fn execute_shr() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0b1011;
        cpu.execute(OpCode::SHR(0));
        assert_eq!(cpu.v_registers[0], 0b0101);
        assert_eq!(cpu.v_registers[0xF], 1);
        cpu.v_registers[0] = 0b1010;
        cpu.execute(OpCode::SHR(0));
        assert_eq!(cpu.v_registers[0], 0b0101);
        assert_eq!(cpu.v_registers[0xF], 0);
    } */
    #[test]
    fn execute_subn() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0b10;
        cpu.v_registers[1] = 0b11;
        cpu.execute(OpCode::SUBN(0, 1));
        assert_eq!(cpu.v_registers[0], 1);
        assert_eq!(cpu.v_registers[0xF], 0);
        cpu.v_registers[0] = 0b11;
        cpu.v_registers[1] = 0b10;
        cpu.execute(OpCode::SUBN(0, 1));
        assert_eq!(cpu.v_registers[0], 255);
        assert_eq!(cpu.v_registers[0xF], 1);
    }
    //TODO: update tests for new implementation
/*     #[test]
    fn execute_shl() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0b10110000;
        cpu.execute(OpCode::SHL(0));
        assert_eq!(cpu.v_registers[0], 0b01100000);
        assert_eq!(cpu.v_registers[0xF], 1);
        cpu.v_registers[0] = 0b00110000;
        cpu.execute(OpCode::SHL(0));
        assert_eq!(cpu.v_registers[0], 0b01100000);
        assert_eq!(cpu.v_registers[0xF], 0);
    } */
    #[test]
    fn execute_sne_xy() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0x10;
        cpu.v_registers[1] = 0x10;
        cpu.execute(OpCode::SkipNotEqualXY(0, 1));
        assert_eq!(cpu.pc, 0x0);
        cpu.v_registers[0] = 0x10;
        cpu.v_registers[1] = 0x11;
        cpu.execute(OpCode::SkipNotEqualXY(0, 1));
        assert_eq!(cpu.pc, 0x2);
    }
    #[test]
    fn execute_jp_v0() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0x10;
        cpu.execute(OpCode::JumpPlusV0(0x10));
        assert_eq!(cpu.pc, 0x20);
    }
    #[test]
    fn execute_rand() {
        let mut cpu = Chip8::new();
        cpu.execute(OpCode::RAND(0, 0xFF));
        //assert_ne!(cpu.v_registers[0], 0);
    }
    #[test]
    fn execute_return() {
        let mut cpu = Chip8::new();
        cpu.stack_pointer = 1;
        cpu.address_stack.push(0xF0);
        cpu.execute(OpCode::RET);
        assert_eq!(cpu.stack_pointer, 0);
        assert_eq!(cpu.pc, 0xF0);
    }
    #[test]
    fn execute_add_vx_to_i() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0x10;
        cpu.i_reg = 0x10;
        cpu.execute(OpCode::AddVxToI(0));
        assert_eq!(cpu.i_reg, 0x20);
    }
    #[test]
    fn execute_save_bcd() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 123;
        cpu.i_reg = 0x10;
        cpu.execute(OpCode::SaveBCD(0));
        assert_eq!(cpu.ram.read_byte(0x10), 1);
        assert_eq!(cpu.ram.read_byte(0x11), 2);
        assert_eq!(cpu.ram.read_byte(0x12), 3);
    }
    #[test]
    fn execute_store_v0_to_vx() {
        let mut cpu = Chip8::new();
        cpu.v_registers[0] = 0x10;
        cpu.v_registers[1] = 0x11;
        cpu.v_registers[2] = 0x12;
        cpu.v_registers[3] = 0x13;
        cpu.i_reg = 0x10;
        cpu.execute(OpCode::StoreV0ToVx(4));
        assert_eq!(cpu.ram.read_byte(0x10), 0x10);
        assert_eq!(cpu.ram.read_byte(0x11), 0x11);
        assert_eq!(cpu.ram.read_byte(0x12), 0x12);
        assert_eq!(cpu.ram.read_byte(0x13), 0x13);
    }
    #[test]
    fn execute_read_v0_to_vx() {
        let mut cpu = Chip8::new();
        cpu.ram.write_byte(0x10, 0x10);
        cpu.ram.write_byte(0x11, 0x11);
        cpu.ram.write_byte(0x12, 0x12);
        cpu.ram.write_byte(0x13, 0x13);
        cpu.ram.write_byte(0x14, 0x14);
        cpu.ram.write_byte(0x15, 0x15);
        cpu.i_reg = 0x10;
        cpu.execute(OpCode::ReadV0ToVx(6));
        assert_eq!(cpu.v_registers[0], 0x10);
        assert_eq!(cpu.v_registers[1], 0x11);
        assert_eq!(cpu.v_registers[2], 0x12);
        assert_eq!(cpu.v_registers[3], 0x13);
        assert_eq!(cpu.v_registers[4], 0x14);
        assert_eq!(cpu.v_registers[5], 0x15);
    }
}