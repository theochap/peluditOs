use crate::kmalloc::KMalloc;
use crate::physical_alloc::buddy::BuddyAllocator;
use crate::physical_alloc::memmap::{MemoryMap, MemoryMapEntry, MemoryMapKind};
use crate::physical_alloc::memzone::{MemZone512, MemZoneExt};
use crate::utils::NonZero;

/// Tries to allocate a kbox on an existing memzone. Ensure that the kbox is
/// allocated in the memzone and that the data is stored correctly.
#[test]
fn test_allocate_memzone() {
    // Let's allocate a 1KB memzone.
    const MEMZONE_SIZE: usize = 1024;
    let mut mem = vec![0_u8; MEMZONE_SIZE];

    // Let's get the address of the memzone.
    let memzone_addr = mem.as_mut_ptr() as usize;

    // Let's create the kernel memory allocator and set it for the memzone.
    let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + MEMZONE_SIZE);

    let string = "hello world!";
    let expected_value = NonZero::NonZero(string);

    // Let's allocate a kernel box with some data in it.
    let kbox = kmalloc.new_box(string).unwrap();

    let mem_ptr = mem.as_ptr() as *const NonZero<&str>;
    let mem_bytes: &NonZero<&str> = unsafe { &*mem_ptr };

    assert_eq!(mem_bytes, &expected_value);
    assert_eq!(*kbox.0, expected_value);
}

/// Tries to allocate a memzone using the buddy allocator.
#[test]
fn test_buddy_allocator() {
    // Let's allocate a 1KB memzone.
    const MEMZONE_SIZE: usize = 2048;
    const NUM_PAGES: usize = 4 * 512 * MEMZONE_SIZE;
    let mut mem = vec![0_u8; NUM_PAGES * MEMZONE_SIZE];

    // Let's get the address of the memzone.
    let memzone_addr = mem.as_mut_ptr() as usize;

    // Let's create the kernel memory allocator and set it for the memzone.
    let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + MEMZONE_SIZE);

    let memmap = MemoryMap::new(
        MemoryMapEntry {
            base_addr: memzone_addr,
            length: NUM_PAGES * MEMZONE_SIZE,
            kind: MemoryMapKind::Usable,
        },
        &mut kmalloc,
    )
    .unwrap();

    // Now create a buddy allocator. It should be able to hold 4 full zones.
    let mut buddy_allocator = BuddyAllocator::<MEMZONE_SIZE>::new(&mut kmalloc, memmap).unwrap();

    let mut iterator = buddy_allocator.pages.iter();
    let first_page = iterator.next().unwrap();
    assert!(first_page.zone.is_full());
    assert_eq!(first_page.zone.size(), 512 * MEMZONE_SIZE);

    assert_eq!(iterator.next(), None);

    // Let's allocate the full second zone.
    buddy_allocator.alloc::<MemZone512>(&mut kmalloc).unwrap();

    // Iterate over the pages and ensure that the second page is full.
    let mut iterator = buddy_allocator.pages.iter();
    let first_page = iterator.next().unwrap();
    assert!(first_page.zone.is_full());
    assert_eq!(first_page.zone.size(), 512 * MEMZONE_SIZE);

    let second_page = iterator.next().unwrap();
    assert!(second_page.zone.is_full());
    assert_eq!(second_page.zone.size(), 512 * MEMZONE_SIZE);

    assert_eq!(iterator.next(), None);
}
