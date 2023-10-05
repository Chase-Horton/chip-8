use std::fs;
const FONT_START_ADDRESS: u16 = 0x00;
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
pub struct EmulatedRam {
    pub data: [u8; 0x1000 as usize], // 4096 bytes of memory
}
impl EmulatedRam {
    pub fn new() -> Self {
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

    pub fn read_byte(&self, address: u16) -> u8 {
        if address > 0xFFF {
            panic!("Ram: tried to access memory out of bounds")
        }
        self.data[address as usize]
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    fn load_program(&mut self, data: Vec<u8>) {
        let mut counter = 0x200;
        for byte in data {
            self.write_byte(counter, byte);
            counter += 1;
        }
    }

    pub fn load_program_from_file(&mut self, file_path: &str) {
        let contents = fs::read(file_path).expect("failed to open program from file");
        self.load_program(contents);
    }
}
#[cfg(test)]
mod test {
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
}
