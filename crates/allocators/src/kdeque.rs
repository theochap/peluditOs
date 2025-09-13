//! Kernel double ended queue.

use core::ops::Deref;

use crate::{
    kmalloc::{Error, KMalloc},
    krc::KRefCell,
};

pub struct KDeque<T: 'static>(Option<KDequePtrs<T>>);

impl<T: 'static> Deref for KDeque<T> {
    type Target = Option<KDequePtrs<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> KDeque<T> {
    /// Creates a new empty deque.
    pub fn new() -> Self {
        Self(None)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let inner = self.0.as_mut()?;

        let curr_head = inner.head.clone();
        let next_head = curr_head.get().next.clone();

        if let Some(new_head) = next_head {
            // Remove the current head from the list.
            new_head.replace(|mut h| {
                h.prev = None;
                h
            });

            inner.head = new_head;
        } else {
            self.0 = None;
        }

        Some(curr_head.take().take().value)
    }

    pub fn push_front(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), Error> {
        let Some(inner) = self.0.as_mut() else {
            let ptr = KDequePtrs::new(kbox_maker, value)?;
            self.0 = Some(ptr);
            return Ok(());
        };

        inner.push_front(kbox_maker, value)?;

        Ok(())
    }

    pub fn push_back(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), Error> {
        let Some(inner) = self.0.as_mut() else {
            let ptr = KDequePtrs::new(kbox_maker, value)?;
            self.0 = Some(ptr);
            return Ok(());
        };

        inner.push_back(kbox_maker, value)?;

        Ok(())
    }
}

pub struct KDequeIter<'a, T: 'static>(Option<&'a KDequeNodeRef<T>>);

impl<'a, T: 'static> KDequeIter<'a, T> {
    pub fn new(deque: &'a KDeque<T>) -> Self {
        Self(deque.0.as_ref().map(|ptr| &ptr.head))
    }
}

impl<'a, T: 'static> Iterator for KDequeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0?;
        let next_val = next.get().next.as_ref();

        self.0 = next_val;

        let curr_val = &next.get().value;

        Some(curr_val)
    }
}

pub type KDequeNodeRef<T> = KRefCell<KDequeNode<T>>;

pub struct KDequePtrs<T: 'static> {
    head: KDequeNodeRef<T>,
    tail: KDequeNodeRef<T>,
}

impl<T: 'static> KDequePtrs<T> {
    pub fn new(kbox_maker: &mut KMalloc, value: T) -> Result<Self, Error> {
        let ptr: KDequeNodeRef<T> = kbox_maker.new_rc(KDequeNode::new(value).into())?;

        Ok(Self {
            head: ptr.clone(),
            tail: ptr,
        })
    }

    pub fn push_front(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), Error> {
        let curr_head = self.head.clone();

        let new_head: KDequeNodeRef<T> = kbox_maker.new_rc(
            KDequeNode {
                value,
                next: Some(curr_head.clone()),
                prev: None,
            }
            .into(),
        )?;

        curr_head.replace(|mut h| {
            h.prev = Some(new_head.clone());
            h
        });

        self.head = new_head;

        Ok(())
    }

    pub fn push_back(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), Error> {
        let curr_tail = self.tail.clone();

        let new_tail: KDequeNodeRef<T> = kbox_maker.new_rc(
            KDequeNode {
                value,
                next: None,
                prev: Some(curr_tail.clone()),
            }
            .into(),
        )?;

        curr_tail.replace(|mut t| {
            t.next = Some(new_tail.clone());
            t
        });

        self.tail = new_tail;

        Ok(())
    }
}

pub struct KDequeNode<T: 'static> {
    value: T,
    next: Option<KDequeNodeRef<T>>,
    prev: Option<KDequeNodeRef<T>>,
}

impl<T: 'static> KDequeNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            next: None,
            prev: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOC_SIZE: usize = 1 << 10;

    #[test]
    fn test_kdeque_push() {
        // Spawn a new kmalloc to allocate the tables.
        let mut mem = vec![0_u8; ALLOC_SIZE];

        // Let's get the address of the memzone.
        let memzone_addr = mem.as_mut_ptr() as usize;

        // Let's create the kernel memory allocator and set it for the memzone.
        let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + ALLOC_SIZE);

        let mut deque = KDeque::new();
        deque.push_front(&mut kmalloc, 1).unwrap();
        deque.push_back(&mut kmalloc, 2).unwrap();
        deque.push_front(&mut kmalloc, 3).unwrap();
        deque.push_back(&mut kmalloc, 4).unwrap();

        let mut iter = KDequeIter::new(&deque);
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_kdeque_pop() {
        // Spawn a new kmalloc to allocate the tables.
        let mut mem = vec![0_u8; ALLOC_SIZE];

        // Let's get the address of the memzone.
        let memzone_addr = mem.as_mut_ptr() as usize;

        // Let's create the kernel memory allocator and set it for the memzone.
        let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + ALLOC_SIZE);

        let mut deque = KDeque::new();
        deque.push_front(&mut kmalloc, 1).unwrap();
        deque.push_back(&mut kmalloc, 2).unwrap();
        deque.push_front(&mut kmalloc, 3).unwrap();
        deque.push_back(&mut kmalloc, 4).unwrap();

        assert_eq!(deque.pop_front(), Some(3));
        assert_eq!(deque.pop_front(), Some(1));
        assert_eq!(deque.pop_front(), Some(2));
        assert_eq!(deque.pop_front(), Some(4));
        assert_eq!(deque.pop_front(), None);
    }
}
