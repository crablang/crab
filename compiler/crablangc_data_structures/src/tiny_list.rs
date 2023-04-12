//! A singly-linked list.
//!
//! Using this data structure only makes sense under very specific
//! circumstances:
//!
//! - If you have a list that rarely stores more than one element, then this
//!   data-structure can store the element without allocating and only uses as
//!   much space as an `Option<(T, usize)>`. If T can double as the `Option`
//!   discriminant, it will even only be as large as `T, usize`.
//!
//! If you expect to store more than 1 element in the common case, steer clear
//! and use a `Vec<T>`, `Box<[T]>`, or a `SmallVec<T>`.

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct TinyList<T> {
    head: Option<Element<T>>,
}

impl<T: PartialEq> TinyList<T> {
    #[inline]
    pub fn new() -> TinyList<T> {
        TinyList { head: None }
    }

    #[inline]
    pub fn new_single(data: T) -> TinyList<T> {
        TinyList { head: Some(Element { data, next: None }) }
    }

    #[inline]
    pub fn insert(&mut self, data: T) {
        self.head = Some(Element { data, next: self.head.take().map(Box::new) });
    }

    #[inline]
    pub fn remove(&mut self, data: &T) -> bool {
        self.head = match &mut self.head {
            Some(head) if head.data == *data => head.next.take().map(|x| *x),
            Some(head) => return head.remove_next(data),
            None => return false,
        };
        true
    }

    #[inline]
    pub fn contains(&self, data: &T) -> bool {
        let mut elem = self.head.as_ref();
        while let Some(e) = elem {
            if &e.data == data {
                return true;
            }
            elem = e.next.as_deref();
        }
        false
    }
}

#[derive(Clone)]
struct Element<T> {
    data: T,
    next: Option<Box<Element<T>>>,
}

impl<T: PartialEq> Element<T> {
    fn remove_next(mut self: &mut Self, data: &T) -> bool {
        loop {
            match self.next {
                Some(ref mut next) if next.data == *data => {
                    self.next = next.next.take();
                    return true;
                }
                Some(ref mut next) => self = next,
                None => return false,
            }
        }
    }
}
