use crate::{
    kbox::KBox,
    kmalloc::{Error, KMalloc},
};

struct KStackInner<T: 'static> {
    head: T,
    tail: Option<KStack<T>>,
}

pub struct KStack<T: 'static>(KBox<KStackInner<T>>);

impl<T: 'static> KStack<T> {
    pub fn new(kbox_maker: &mut KMalloc, head: T) -> Result<Self, Error> {
        let list = KMalloc::new_box(kbox_maker, KStackInner { head, tail: None })?;
        Ok(Self(list))
    }

    pub fn new_with_tail(kbox_maker: &mut KMalloc, head: T, tail: Self) -> Result<Self, Error> {
        let list = KMalloc::new_box(
            kbox_maker,
            KStackInner {
                head,
                tail: Some(tail),
            },
        )?;
        Ok(Self(list))
    }

    pub fn push(self, kbox_maker: &mut KMalloc, value: T) -> Result<Self, Error> {
        let new_list = KStack::new_with_tail(kbox_maker, value, self)?;
        Ok(new_list)
    }

    pub fn head(&self) -> &T {
        &self.0.head
    }

    pub fn head_mut(&mut self) -> &mut T {
        &mut self.0.head
    }

    pub fn tail(&self) -> Option<&Self> {
        self.0.tail.as_ref()
    }

    pub fn tail_mut(&mut self) -> Option<&mut Self> {
        self.0.tail.as_mut()
    }

    pub fn pop(self) -> (T, Option<Self>) {
        let inner = self.0.take();
        let head = inner.head;
        let tail = inner.tail;
        (head, tail)
    }

    pub fn apply<F: Fn(&mut T)>(&mut self, f: F) {
        let mut next = Some(self);

        while let Some(next_list) = next {
            f(next_list.head_mut());
            next = next_list.tail_mut();
        }
    }

    pub fn apply_until<Out, Err, F: FnMut(&mut T) -> Result<Option<Out>, Err>>(
        &mut self,
        mut f: F,
    ) -> Result<Option<Out>, Err> {
        let mut next = Some(self);

        while let Some(next_list) = next {
            match f(next_list.head_mut()) {
                Err(err) => return Err(err),
                Ok(Some(out)) => return Ok(Some(out)),
                Ok(None) => next = next_list.tail_mut(),
            }
        }

        Ok(None)
    }
}

pub struct KListIter<'a, T: 'static>(Option<&'a KStack<T>>);

impl<'a, T: 'static> Iterator for KListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let list = self.0?;
        let head = list.head();
        self.0 = list.tail();
        Some(head)
    }
}
