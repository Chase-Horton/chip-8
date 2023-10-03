#![allow(unused)]
struct EmulatedRam {
    data: [u8; 0x1000 as usize], // 4096 bytes of memory
}
static FONT_START_ADDRESS: u16 = 0x50;
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
impl EmulatedRam {
    fn new() -> Self {
        let mut ram = EmulatedRam { data: [0; 0x1000] };
        let fonts = load_fonts();
        let mut address = FONT_START_ADDRESS;
        for font in fonts.iter() {
            println!("{:#02x}", font);
            ram.write_byte(address, *font);
            address += 1
        }
        ram
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
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
    // fn put_pixel(&mut self, x: u8, y: u8, pix: bool) -> Result<bool, &str> {
    //     if x < 64 && y < 32 {
    //         self.pixels[y as usize][x as usize] = pix;
    //         Ok(pix)
    //     } else {
    //         Err("Attempting to put pixel out of bounds")
    //     }
    // }
}
struct DelayTimer {
    val: u8,
}
impl DelayTimer {
    fn new() -> DelayTimer {
        DelayTimer { val: 0 }
    }
}
struct Chip8 {
    pc: u16,
    i_reg: u16,
    address_stack: Vec<u16>,
    delay_timer: DelayTimer,
    v_registers: Vec<u8>,
    screen: EmulatedScreen,
    ram: EmulatedRam,
}
impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            pc: 0,
            i_reg: 0,
            address_stack: Vec::new(),
            delay_timer: DelayTimer::new(),
            v_registers: Vec::new(),
            screen: EmulatedScreen::new(),
            ram: EmulatedRam::new(),
        }
    }
}
fn main() {
    let mut ram = EmulatedRam::new();

    // Write a value to address 0x0A
    ram.write_byte(0xFFF, 0x42);

    // Read the value at address 0x0A
    let value = ram.read_byte(0xFFF);

    println!("Value at address 0xFFF: 0x{:02X}", value);

    // Screen
    let mut screen = EmulatedScreen::new();
    screen.pixels[20][45] = true;
    println!("{:?}", screen.pixels[20][45]);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ram_read() {
        let ram = EmulatedRam::new();
        for i in 0x80..=0xFFF {
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
}

