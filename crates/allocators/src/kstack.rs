use crate::{
    kbox::KBox,
    kmalloc::{Error, KMalloc},
};

struct KStackInner<T: 'static> {
    head: T,
    tail: KStack<T>,
}

pub struct KStack<T: 'static>(Option<KBox<KStackInner<T>>>);

impl<T: 'static> KStack<T> {
    pub fn new(kbox_maker: &mut KMalloc, head: T) -> Result<Self, Error> {
        let list = KMalloc::new_box(
            kbox_maker,
            KStackInner {
                head,
                tail: Self(None),
            },
        )?;
        Ok(Self(Some(list)))
    }

    pub fn new_with_tail(kbox_maker: &mut KMalloc, head: T, tail: Self) -> Result<Self, Error> {
        let list = KMalloc::new_box(kbox_maker, KStackInner { head, tail })?;
        Ok(Self(Some(list)))
    }

    pub fn push(&mut self, kbox_maker: &mut KMalloc, value: T) -> Result<(), Error> {
        match self.0.as_mut() {
            Some(list) => {
                list.try_map_inner(|inner| {
                    let inner = kbox_maker.new_box(inner)?;

                    Ok(KStackInner {
                        head: value,
                        tail: Self(Some(inner)),
                    })
                })?;
            }
            None => {
                let list = kbox_maker.new_box(KStackInner {
                    head: value,
                    tail: Self(None),
                })?;
                *self = Self(Some(list));
            }
        }
        Ok(())
    }

    pub fn head(&self) -> Option<&T> {
        self.0.as_ref().map(|inner| &inner.head)
    }

    pub fn head_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut().map(|inner| &mut inner.head)
    }

    pub fn tail(&self) -> Option<&Self> {
        self.0.as_ref().map(|inner| &inner.tail)
    }

    pub fn tail_mut(&mut self) -> Option<&mut Self> {
        self.0.as_mut().map(|inner| &mut inner.tail)
    }

    pub fn pop(&mut self) -> Option<T> {
        let inner = self.0.as_mut()?;

        let head = inner
            .try_map_inner_with_output(|mut inner| {
                let tail = inner.tail.0.take().ok_or(())?.take();

                Ok::<_, ()>((tail, inner.head))
            })
            .ok()?;

        Some(head)
    }

    pub fn apply<F: Fn(&mut T)>(&mut self, f: F) {
        let mut next = Some(self);

        while let Some(next_list) = next
            && let Some(head) = next_list.head_mut()
        {
            f(head);
            next = next_list.tail_mut();
        }
    }

    pub fn apply_until<Out, Err, F: FnMut(&mut T) -> Result<Option<Out>, Err>>(
        &mut self,
        mut f: F,
    ) -> Result<Option<Out>, Err> {
        let mut next = Some(self);

        while let Some(next_list) = next
            && let Some(head) = next_list.head_mut()
        {
            match f(head) {
                Err(err) => return Err(err),
                Ok(Some(out)) => return Ok(Some(out)),
                Ok(None) => next = next_list.tail_mut(),
            }
        }

        Ok(None)
    }

    pub fn iter(&'_ self) -> KListIter<'_, T> {
        KListIter(Some(self))
    }
}

pub struct KListIter<'a, T: 'static>(Option<&'a KStack<T>>);

impl<'a, T: 'static> Iterator for KListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let list = self.0?;
        let head = list.head()?;
        self.0 = list.tail();
        Some(head)
    }
}
