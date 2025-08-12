#![no_std]
#![no_main]

use core::fmt::Write;

use pelu_graphics::{clear_screen, kprintln};

#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    // Clear the screen...
    clear_screen();

    kprintln!("Booted in long mode!");

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
