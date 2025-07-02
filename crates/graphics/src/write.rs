use core::fmt::Write;

use crate::vga::{Char, Color, Coord, VGA_HEIGHT, VGA_WIDTH, VgaState};

impl VgaState {
    fn clear_screen(&mut self) {
        for row in 0..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                self.vga_buffer
                    .put_char(Char::new(0x20, Color::White), Coord { row, column: col });
            }
        }

        self.curr_coord.row = 0;
        self.curr_coord.column = 0;
    }

    #[inline(always)]
    fn write_char(&mut self, char: char) {
        if char == '\x00' {
            return;
        }

        if self.curr_coord.row >= VGA_HEIGHT {
            self.clear_screen();
        }

        match char {
            '\n' => self.new_line(),
            '\r' => self.curr_coord.column = 0,
            '\t' => self.curr_coord.column += 4,
            _ => {
                // Ensure the character is a valid ASCII character
                let char = if ('\x20'..='\x7F').contains(&char) {
                    Char::new(char as u8, Color::White)
                } else {
                    // If the character is not a valid ASCII character we print a white box.
                    Char::new(0xFE, Color::White)
                };

                self.put_char(char);
            }
        }
    }

    #[inline(always)]
    fn put_char(&mut self, char: Char) {
        self.vga_buffer.put_char(char, self.curr_coord);

        if self.curr_coord.column >= VGA_WIDTH {
            self.curr_coord.column = 0;
            self.curr_coord.row += 1;
        }

        if self.curr_coord.row >= VGA_HEIGHT {
            self.curr_coord.row = 0;
        }

        self.curr_coord.column += 1;
    }

    #[inline(always)]
    fn new_line(&mut self) {
        self.curr_coord.column = 0;
        self.curr_coord.row += 1;
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
