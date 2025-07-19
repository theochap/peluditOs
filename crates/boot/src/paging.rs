//! Contains information related to the paging data structures of the kernel.
//!
//! This uses a IA32 4-level paging structure compatible with the x86_64 architecture.
//!
//! At boot time, the kernel will use identity mapping for the first 2 * 512 MiB (1GiB) of physical memory.
//!
//! To do that we use the large page feature to map level 2 pages to 2MiB pages.

use core::arch::asm;
use core::fmt::Write;
use core::ops::IndexMut;

use pelu_graphics::kprintln;

const PAGE_TABLE_SIZE: usize = 512;

/// The size of a large page in bytes. Alias for 2MiB.
const LARGE_PAGE_SIZE: u64 = 0x200000;

#[repr(C)]
pub struct PageTable {
    pub entries: [u64; PAGE_TABLE_SIZE],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [0; PAGE_TABLE_SIZE],
        }
    }

    /// Maps a page to the physical address and flags provided.
    ///
    /// Note: we don't have a heap yet so we're using raw assembly to set the page table entry.
    ///
    /// ## Panics
    /// We don't need to set more than 32 bits for the physical address + flags because we're still in 32-bits compatibility mode and
    /// we're only mapping the first 1GiB of physical memory. This method will panic if the physical address + flags
    /// is greater than 32 bits.
    pub fn map_page(&mut self, entry: usize, phys_addr: u64, flags: u64) {
        let index = self.entries.index_mut(entry) as *mut u64;
        let full_value = phys_addr | flags;
        let lower_32: u32 = full_value
            .try_into()
            .expect("Physical address + flags is greater than 32 bits");

        unsafe {
            asm!("
            push eax
            mov eax, {0:e}
            mov [eax], {1:e}    
            pop eax", 
            in(reg) index,
            in(reg) lower_32,
            );
        }
    }

    /// Maps all entries in the page table to the physical address and flags provided by the map function.
    pub fn map_entries(&mut self, map_fn: impl Fn(usize) -> (u64, u64)) {
        for entry in 0..PAGE_TABLE_SIZE {
            let (phys_addr, flags) = map_fn(entry);
            self.map_page(entry, phys_addr, flags);
        }
    }
}

#[repr(C)]
#[repr(align(4096))]
pub struct IdentityPaging {
    pub level_4: PageTable,
    pub level_3: PageTable,
    pub level_2: PageTable,
}

impl IdentityPaging {
    pub const fn new() -> Self {
        Self {
            level_4: PageTable::new(),
            level_3: PageTable::new(),
            level_2: PageTable::new(),
        }
    }
}

#[repr(u64)]
pub enum Flags {
    Present = 1 << 0,
    Writable = 1 << 1,
    User = 1 << 2,
    WriteThrough = 1 << 3,
    CacheDisabled = 1 << 4,
    Accessed = 1 << 5,
    Dirty = 1 << 6,
    LargePage = 1 << 7,
    Global = 1 << 8,
    NoExecute = 1 << 63,
}

#[unsafe(link_section = ".bss")]
pub static mut IDENT_PAGING: IdentityPaging = IdentityPaging::new();

pub fn setup_identity_paging() {
    kprintln!("=== Identity Paging Setup ===");
    // SAFETY: We are not mutating the static variable IDENT_PAGING from multiple threads (because the kernel is single-threaded at boot time).
    unsafe {
        let level_3_addr = &raw const IDENT_PAGING.level_3 as u64;
        let level_2_addr = &raw const IDENT_PAGING.level_2 as u64;
        let pages = &raw mut IDENT_PAGING;

        // Map the level 4 page table to the physical address of the level 3 page table. Set the present and writable flags.
        (*pages).level_4.map_page(
            0,
            level_3_addr,
            Flags::Present as u64 | Flags::Writable as u64,
        );

        // Map the level 3 page table to the physical address of the level 2 page table. Set the present and writable flags.
        (*pages).level_3.map_page(
            0,
            level_2_addr,
            Flags::Present as u64 | Flags::Writable as u64,
        );

        // Map the level 2 page tables to the first 1GiB of physical memory. Set the present and writable flags.
        (*pages).level_2.map_entries(|entry| {
            let phys_addr = entry as u64 * LARGE_PAGE_SIZE;
            (
                phys_addr,
                Flags::Present as u64 | Flags::Writable as u64 | Flags::LargePage as u64,
            )
        });
    }
    kprintln!("Identity paging setup successful!");
    kprintln!();
}
