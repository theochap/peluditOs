use crate::kmalloc::KMalloc;
use crate::utils::NonZero;

const LARGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

#[test]
fn test_allocate_memzone() {
    // Let's allocate a 32MB memzone.
    const MEMZONE_SIZE: usize = 1024;
    let mut mem = vec![0_u8; MEMZONE_SIZE];

    // Let's get the address of the memzone.
    let memzone_addr = mem.as_mut_ptr() as usize;

    // Let's create the kernel memory allocator and set it for the memzone.
    let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + LARGE_PAGE_SIZE);

    let string = "hello world!";
    let expected_value = NonZero::NonZero(string);

    // Let's allocate a kernel box with some data in it.
    let kbox = kmalloc.new_box(string).unwrap();

    let mem_ptr = mem.as_ptr() as *const NonZero<&str>;
    let mem_bytes: &NonZero<&str> = unsafe { &*mem_ptr };

    assert_eq!(mem_bytes, &expected_value);
    assert_eq!(*kbox.0, expected_value);
}
