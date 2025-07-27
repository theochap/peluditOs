#![no_std]

//! Graphics library for peluditOS

mod vga;
mod write;

pub use vga::VGA_STATE;
pub use write::clear_screen;
