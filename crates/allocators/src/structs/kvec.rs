//! Kernel vector.

use core::mem;

use crate::{
    kmalloc::{Error, KMalloc},
    utils::NonZero,
};

pub struct KVec<T: 'static> {
    len: usize,
    capacity: usize,
    inner: &'static mut [NonZero<T>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KVecError {
    KMalloc(Error),
    CapacityReached,
    IndexOutOfBounds,
}

impl From<Error> for KVecError {
    fn from(error: Error) -> Self {
        KVecError::KMalloc(error)
    }
}

impl<T> KVec<T> {
    /// Creates a new KVec with the given capacity.
    pub fn new(capacity: usize, kbox_maker: &mut KMalloc) -> Result<Self, Error> {
        // Allocate the memory for the inner array.
        let inner = kbox_maker.reserve_array::<NonZero<T>>(capacity)?;

        Ok(Self {
            len: 0,
            capacity,
            inner,
        })
    }

    pub fn insert(&mut self, index: usize, value: T) -> Result<(), KVecError> {
        if index > self.len {
            return Err(KVecError::IndexOutOfBounds);
        }

        if self.len + 1 > self.capacity {
            return Err(KVecError::CapacityReached);
        }

        // Shift the elements to the right.
        for i in (index..self.len).rev() {
            self.inner[i + 1] = self.inner[i].take().into();
        }

        self.inner[index] = value.into();
        self.len += 1;
        Ok(())
    }

    pub fn push_no_resize(&mut self, value: T) -> Result<(), KVecError> {
        if self.len + 1 > self.capacity {
            return Err(KVecError::CapacityReached);
        }

        self.inner[self.len] = value.into();
        self.len += 1;
        Ok(())
    }

    pub fn push(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), Error> {
        if self.len + 1 > self.capacity {
            let new_capacity = self.capacity * 2;
            let new_inner = kbox_maker.reserve_array::<NonZero<T>>(new_capacity)?;
            self.capacity = new_capacity;

            self.inner
                .iter_mut()
                .enumerate()
                .for_each(|(i, value)| new_inner[i] = value.take().into());

            // Copy the old inner array to the new inner array.
            let _ = mem::replace(&mut self.inner, new_inner);
        }

        self.inner[self.len] = value.into();
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        Some(self.inner[self.len].take())
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        Some(self.inner[index].inner_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        Some(self.inner[index].inner_ref_mut())
    }

    pub fn first(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }

        Some(self.inner[0].inner_ref())
    }

    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }

        Some(self.inner[self.len - 1].inner_ref())
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            return None;
        }

        Some(self.inner[self.len - 1].inner_ref_mut())
    }

    pub fn iter(&self) -> KVecIter<T> {
        KVecIter {
            kvec: self,
            index: 0,
        }
    }
}

pub struct KVecIter<'a, T: 'static> {
    kvec: &'a KVec<T>,
    index: usize,
}

impl<'a, T: 'a> Iterator for KVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.kvec.get(self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOC_SIZE: usize = 1 << 10;

    #[test]
    fn test_kvec() {
        // Spawn a new kmalloc to allocate the tables.
        let mut mem = vec![0_u8; ALLOC_SIZE];

        // Let's get the address of the memzone.
        let memzone_addr = mem.as_mut_ptr() as usize;

        // Let's create the kernel memory allocator and set it for the memzone.
        let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + ALLOC_SIZE);

        let mut kvec = KVec::new(10, &mut kmalloc).unwrap();

        kvec.push(&mut kmalloc, 1).unwrap();
        kvec.push(&mut kmalloc, 2).unwrap();
        kvec.push(&mut kmalloc, 3).unwrap();

        assert_eq!(kvec.get(0), Some(&1));
        assert_eq!(kvec.get(1), Some(&2));
        assert_eq!(kvec.get(2), Some(&3));
        assert_eq!(kvec.get(3), None);

        assert_eq!(kvec.pop(), Some(3));
        assert_eq!(kvec.pop(), Some(2));
        assert_eq!(kvec.pop(), Some(1));
        assert_eq!(kvec.pop(), None);

        assert_eq!(kvec.len, 0);
        assert_eq!(kvec.capacity, 10);

        assert_eq!(kvec.get(0), None);
        assert_eq!(kvec.get(1), None);
        assert_eq!(kvec.get(2), None);
    }

    #[test]
    fn test_kvec_resize() {
        // Spawn a new kmalloc to allocate the tables.
        let mut mem = vec![0_u8; ALLOC_SIZE];

        // Let's get the address of the memzone.
        let memzone_addr = mem.as_mut_ptr() as usize;

        // Let's create the kernel memory allocator and set it for the memzone.
        let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + ALLOC_SIZE);

        let mut kvec = KVec::new(3, &mut kmalloc).unwrap();

        kvec.push(&mut kmalloc, 1).unwrap();
        kvec.push(&mut kmalloc, 2).unwrap();
        kvec.push(&mut kmalloc, 3).unwrap();

        // Assert that the capacity and the length are 3.
        assert_eq!(kvec.len, 3);
        assert_eq!(kvec.capacity, 3);

        let err = kvec.push_no_resize(4).unwrap_err();
        assert_eq!(err, KVecError::CapacityReached);

        // Let's try to push a fourth element.
        kvec.push(&mut kmalloc, 4).unwrap();

        // Assert that the capacity is 6.
        assert_eq!(kvec.len, 4);
        assert_eq!(kvec.capacity, 6);

        assert_eq!(kvec.get(0), Some(&1));
        assert_eq!(kvec.get(1), Some(&2));
        assert_eq!(kvec.get(2), Some(&3));
        assert_eq!(kvec.get(3), Some(&4));
    }
}
