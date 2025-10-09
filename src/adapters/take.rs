use core::ptr::Pointee;

use crate::{util::Length, Bulk};

/// A bulk that only delivers the first `n` iterations of `iter`.
///
/// This `struct` is created by the [`take`](Bulk::take) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Take<I, N>
where
    I: Bulk,
    N: Length + ?Sized
{
    iterable: I,
    n: <N as Pointee>::Metadata
}

impl<I, N> Take<I, N>
where
    I: Bulk,
    N: Length + ?Sized
{
    pub fn new(iterable: I, n: <N as Pointee>::Metadata) -> Take<I, N>
    {
        Self { iterable, n }
    }
}
impl<I, N> IntoIterator for Take<I, N>
where
    I: Bulk,
    N: Length + ?Sized
{
    type Item = I::Item;
    type IntoIter = core::iter::Take<I::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { iterable, n } = self;
        iterable.into_iter().take(N::len_metadata(n))
    }
}
impl<I, N> Bulk for Take<I, N>
where
    I: Bulk,
    N: Length + ?Sized
{
    fn len(&self) -> usize
    {
        N::len_metadata(self.n).min(self.iterable.len())
    }
}