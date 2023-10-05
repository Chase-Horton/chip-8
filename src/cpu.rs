mod opcodes;
use opcodes::OpCode;
mod ram;
use ram::EmulatedRam;
mod screen;
use screen::EmulatedScreen;
use std::fs;

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
    i_reg: u16,
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
                    self.v_registers[0xF] =
                        self.screen.write_byte(v_x, v_y + row, spirte_byte_from_mem)
                }
            }
            _ => {}
        }
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
}
