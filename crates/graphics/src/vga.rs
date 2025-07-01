use core::{ops::DerefMut, ptr::write_volatile};

use lazy_static::lazy_static;
use spin::Mutex;

const VGA_BUFFER_ADDRESS: *mut u8 = 0xb8000 as *mut u8;

pub(crate) const VGA_WIDTH: u8 = 80;
pub(crate) const VGA_HEIGHT: u8 = 25;

#[derive(Copy, Clone)]
#[repr(u8)]
pub(crate) enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct Char {
    character: u8,
    color: Color,
}

impl Char {
    pub(crate) const fn new(character: u8, color: Color) -> Self {
        Self { character, color }
    }
}

#[derive(Copy, Clone)]
pub struct Coord {
    pub(crate) row: u8,
    pub(crate) column: u8,
}

#[repr(transparent)]
pub(crate) struct VgaBuffer([[Char; VGA_WIDTH as usize]; VGA_HEIGHT as usize]);

impl VgaBuffer {
    #[inline(always)]
    pub(crate) fn put_char(&mut self, char: Char, coord: Coord) {
        let row = coord.row as usize;
        let column = coord.column as usize;
        let vga_buffer = self.0[row].as_mut_ptr();
        unsafe {
            write_volatile(vga_buffer.add(column), char);
        }
    }
}

pub struct VgaState {
    pub(crate) curr_coord: Coord,
    pub(crate) vga_buffer: &'static mut VgaBuffer,
}

lazy_static! {
    // TODO: Once we have a proper scheduler, use that instead of a spinlock.
    pub static ref VGA_STATE: Mutex<VgaState> = Mutex::new(VgaState {
        curr_coord: Coord { row: 0, column: 0 },
        // SAFETY: We know that the VGA buffer is at 0xb8000 and that the vga buffer size is 80x25
        // so we can safely cast the pointer to a VgaBuffer.
        vga_buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut VgaBuffer) },
    });
}
