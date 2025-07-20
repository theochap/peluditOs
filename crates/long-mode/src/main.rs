#![no_std]
#![no_main]

use core::fmt::Write;

use pelu_graphics::kprintln;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() {
    kprintln!("Booted in long mode!");

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
