use core::fmt::Write;

use crate::vga::{Char, Color, VGA_HEIGHT, VGA_WIDTH, VgaState};

impl VgaState {
    #[inline(always)]
    fn write_char(&mut self, char: char) {
        if self.curr_coord.column >= VGA_WIDTH {
            self.curr_coord.column = 0;
            self.curr_coord.row += 1;
        }
        if self.curr_coord.row >= VGA_HEIGHT {
            self.curr_coord.row = 0;
        }

        if char == '\x00' {
            return;
        }

        if char == '\n' {
            self.curr_coord.column = 0;
            self.curr_coord.row += 1;
        } else if char == '\r' {
            self.curr_coord.column = 0;
        } else if char == '\t' {
            self.curr_coord.column += 4;
        }

        // Ensure the character is a valid ASCII character
        if !('\x20'..='\x7F').contains(&char) {
            return;
        }

        let char = Char::new(char as u8, Color::White);
        self.vga_buffer.put_char(char, self.curr_coord);
        self.curr_coord.column += 1;
    }

    fn clear_screen(&mut self) {
        todo!()
    }

    fn new_line(&mut self) {
        todo!()
    }
}

impl Write for VgaState {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for char in s.chars() {
            self.write_char(char);
        }
        Ok(())
    }
}
