use core::ops::{Deref, DerefMut};

use crate::{
    kmalloc::{Error, KMalloc},
    kstack::KStack,
};

#[derive(Debug, PartialEq, Eq)]
pub struct MemoryMapEntry {
    pub base_addr: usize,
    pub length: usize,
    pub kind: MemoryMapKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MemoryMapKind {
    Usable,
    Reserved,
}

pub struct MemoryMap(pub(crate) KStack<MemoryMapEntry>);

impl From<KStack<MemoryMapEntry>> for MemoryMap {
    fn from(stack: KStack<MemoryMapEntry>) -> Self {
        Self(stack)
    }
}

impl MemoryMap {
    pub fn new(first_entry: MemoryMapEntry, kmalloc: &mut KMalloc) -> Result<Self, Error> {
        let list = KStack::new(kmalloc, first_entry)?;
        Ok(Self(list))
    }
}

impl Deref for MemoryMap {
    type Target = KStack<MemoryMapEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MemoryMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
