// BSL 1.0 License

//! Functionality for dealing with arrays in older versions of Rust.
//!
//! Largely uses `tinyvec`'s `ArrayVec` type to do this.

use core::iter::FusedIterator;
use tinyvec::{Array, ArrayVec, ArrayVecIterator};

/// Map one array to another over a function.
pub(crate) fn map<I, O, F>(input: I, f: F) -> O
where
    F: FnMut(I::Item) -> O::Item,
    I: Array,
    O: Array,
{
    let mut output: ArrayVec<O> = ArrayVec::new();
    for i in IntoIter::new(input).map(f) {
        output.push(i);
    }
    output.into_inner()
}

/// An iterator over an array.
pub(crate) struct IntoIter<Arr: Array> {
    inner: ArrayVecIterator<Arr>,
}

impl<Arr: Array> IntoIter<Arr> {
    /// Iterate over an array.
    #[inline]
    pub(crate) fn new(arr: Arr) -> Self {
        let vec = ArrayVec::from(arr);
        Self {
            inner: vec.into_iter(),
        }
    }
}

impl<Arr: Array> Iterator for IntoIter<Arr> {
    type Item = Arr::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n)
    }
    fn last(self) -> Option<Self::Item> {
        self.inner.last()
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.count()
    }
}
impl<Arr: Array> FusedIterator for IntoIter<Arr> {}
impl<Arr: Array> ExactSizeIterator for IntoIter<Arr> {}
impl<Arr: Array> DoubleEndedIterator for IntoIter<Arr> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n)
    }
}
