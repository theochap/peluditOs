#![no_std]
#![no_main]

use core::fmt::Write;

use pelu_graphics::VGA_STATE;

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    write!(VGA_STATE.lock(), "Booted in long mode!").unwrap();

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
