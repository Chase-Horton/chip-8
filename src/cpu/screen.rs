pub struct EmulatedScreen {
    pixels: [[bool; 64]; 32],
}
impl EmulatedScreen {
    pub fn new() -> Self {
        EmulatedScreen {
            pixels: [[false; 64]; 32],
        }
    }
    pub fn put_pixel(&mut self, x: u8, y: u8, pix: bool) {
        self.pixels[(y % 32) as usize][(x % 64) as usize] = pix;
    }
    pub fn get_pixel(&self, x: u8, y: u8) -> bool {
        self.pixels[y as usize][x as usize]
    }
    pub fn get_screen(&self) -> [[bool; 64]; 32] {
        self.pixels
    }
    pub fn write_byte(&mut self, x: u8, y_plus_row: u8, byte: u8) -> u8 {
        let mut v_f: u8 = 0;
        for col in 0..8 {
            // shift to right and get last bit
            let pixel = (byte >> (7 - col)) & 1;
            // wrap around
            let screen_x = ((x + col) % 64) as usize;
            let screen_y = (y_plus_row % 32) as usize;
            // set flag
            if pixel == 1 {
                if self.pixels[screen_y][screen_x] {
                    v_f = 0x1;
                }
                self.pixels[screen_y][screen_x] ^= true;
            }
        }
        v_f
    }
    pub fn clear(&mut self) {
        for x in 0..64 {
            for y in 0..32 {
                self.pixels[y][x] = false;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chip8;
    // Screen Tests
    #[test]
    fn screen_write_every_pixel() {
        let mut screen = EmulatedScreen::new();
        for y in 0..32 {
            for x in 0..64 {
                screen.put_pixel(x, y, true);
            }
        }
        for y in 0..32 {
            for x in 0..64 {
                assert_eq!(screen.get_pixel(x, y), true);
            }
        }
    }
    #[test]
    fn screen_write_oob() {
        let mut screen = EmulatedScreen::new();
        let bad_res = screen.put_pixel(64, 0, true);
        assert_eq!(screen.get_pixel(0, 0), true);
    }
    //display
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
        chip8.ram.write_byte(0x500, 0b11011001);
        chip8.ram.write_byte(0x501, 0b10101010);
        chip8.i_reg = 0x500;
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
    #[test]
    fn execute_display_offset() {
        let mut chip8 = Chip8::new();
        //test basic display of 1 byte
        chip8.ram.write_byte(0x900, 0b11011001);
        chip8.ram.write_byte(0x901, 0b10101010);
        chip8.i_reg = 0x900;
        chip8.v_registers[0] = 1;
        chip8.v_registers[1] = 5;
        let res = chip8.decode(0xD012);
        chip8.execute(res);

        assert_eq!(chip8.screen.pixels[5][1], true);
        assert_eq!(chip8.screen.pixels[5][2], true);
        assert_eq!(chip8.screen.pixels[5][3], false);
        assert_eq!(chip8.screen.pixels[5][4], true);
        assert_eq!(chip8.screen.pixels[5][5], true);
        assert_eq!(chip8.screen.pixels[5][6], false);
        assert_eq!(chip8.screen.pixels[5][7], false);
        assert_eq!(chip8.screen.pixels[5][8], true);

        assert_eq!(chip8.screen.pixels[6][1], true);
        assert_eq!(chip8.screen.pixels[6][2], false);
        assert_eq!(chip8.screen.pixels[6][3], true);
        assert_eq!(chip8.screen.pixels[6][4], false);
        assert_eq!(chip8.screen.pixels[6][5], true);
        assert_eq!(chip8.screen.pixels[6][6], false);
        assert_eq!(chip8.screen.pixels[6][7], true);
        assert_eq!(chip8.screen.pixels[6][8], false);
    }
}
