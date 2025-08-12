use crate::{
    kmalloc::{Error, KMalloc},
    kstack::KStack,
};

pub struct MemoryMapEntry {
    pub base_addr: usize,
    pub length: usize,
    pub kind: MemoryMapKind,
}

pub enum MemoryMapKind {
    Usable,
    Reserved,
}

pub struct MemoryMap(pub(crate) KStack<MemoryMapEntry>);

impl MemoryMap {
    pub fn new(first_entry: MemoryMapEntry, kmalloc: &mut KMalloc) -> Result<Self, Error> {
        let list = KStack::new(kmalloc, first_entry)?;
        Ok(Self(list))
    }

    pub fn push(self, entry: MemoryMapEntry, kmalloc: &mut KMalloc) -> Result<Self, Error> {
        let list = self.0.push(kmalloc, entry)?;
        Ok(Self(list))
    }
}
