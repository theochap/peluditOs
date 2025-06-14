#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn _start() {
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
