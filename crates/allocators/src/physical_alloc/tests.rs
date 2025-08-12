use std::collections::HashMap;
use std::vec::Vec;

use crate::kmalloc::KMalloc;

const LARGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

#[test]
fn test_allocate_memzone() {
    // Let's allocate a 32MB memzone.
    const MEMZONE_SIZE: usize = 16 * LARGE_PAGE_SIZE;
    let mem = Vec::<u8>::with_capacity(MEMZONE_SIZE);

    // Let's get the address of the memzone.
    let memzone_addr = mem.as_ptr() as usize;

    // Let's create the kernel memory allocator and set it for the memzone.
    let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + LARGE_PAGE_SIZE);

    // Let's allocate a kernel box with some data in it.
    let kbox = kmalloc.new_box("hello world").unwrap();

    // Let's print the data in the kernel box.
    println!("{:?}", kbox);
}
