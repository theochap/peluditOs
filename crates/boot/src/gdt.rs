use core::{arch::asm, fmt::Write};

use pelu_graphics::kprintln;

pub struct Gdt {
    /// The first entry is the null entry.
    /// The second entry is the code segment.
    /// Data segments are not necessary in 64-bit mode.
    pub entries: [u64; 2],
}

pub struct GdtPtr {
    len: u16,
    ptr: u64,
}

/// The flags we're using for the GDT entries.
///
/// Those are not all the GDT flags.
/// <https://wiki.osdev.org/Global_Descriptor_Table>
#[repr(u64)]
enum GDTFlags {
    Executable = 1 << 43,
    DescriptorTable = 1 << 44,
    Present = 1 << 47,
    LongMode = 1 << 53,
}

impl Gdt {
    pub const fn new() -> Self {
        let mut gdt = Self { entries: [0; 2] };

        // Set the code segment.
        gdt.entries[1] = GDTFlags::Executable as u64
            | GDTFlags::DescriptorTable as u64
            | GDTFlags::Present as u64
            | GDTFlags::LongMode as u64;

        gdt
    }

    pub fn load_gdt(&self) {
        kprintln!("=== GDT Setup ===");
        let gdt_ptr = GdtPtr {
            // The length is the size of the GDT in bytes minus 1.
            len: (self.entries.len() * size_of::<u64>()) as u16 - 1,
            ptr: self.entries.as_ptr() as u64,
        };

        kprintln!(
            "Loading GDT with length {:?} and address {:X}...",
            gdt_ptr.len,
            gdt_ptr.ptr
        );

        unsafe {
            asm!("lgdt [{0:e}]", in(reg) &gdt_ptr);
        }

        kprintln!("GDT loaded successfully.");
    }
}

/// The GDT is stored in the .rodata section because it's read-only.
/// We're need to support it for legacy reasons.
#[unsafe(link_section = ".rodata")]
pub static GDT: Gdt = Gdt::new();
