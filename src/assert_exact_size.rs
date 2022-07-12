// BSL 1.0 License

use core::iter::FusedIterator;

/// For some reason `tinyvec`'s iterators don't implement `ExactSizedIterator`,
/// despite having an exact size.
#[repr(transparent)]
pub(crate) struct AssertExactSize<I>(pub(crate) I);

impl<I: Iterator> Iterator for AssertExactSize<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.0.last()
    }
}
impl<I: FusedIterator> FusedIterator for AssertExactSize<I> {}
impl<I: DoubleEndedIterator> DoubleEndedIterator for AssertExactSize<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}
impl<I: Iterator> ExactSizeIterator for AssertExactSize<I> {}
