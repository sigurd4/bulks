use core::ptr::Pointee;

use crate::{util::Length, Bulk, ContainedIntoIter, IntoBulk, IntoContained, StaticBulk};

/// Creates a bulk that only delivers the first `n` iterations of `iterable`.
#[allow(invalid_type_param_default)]
pub const fn take<I, N = [<I as IntoIterator>::Item]>(iterable: I, n: <N as Pointee>::Metadata) -> Take<
    <<I as IntoContained>::IntoContained as IntoBulk>::IntoBulk,
    N
>
where
    I: ~const IntoContained,
    N: Length<Elem = I::Item> + ?Sized
{
    unsafe {
        Take::new(iterable.into_contained().into_bulk(), n)
    }
}

/// A bulk that only delivers the first `n` iterations of `bulk`.
///
/// This `struct` is created by the [`take`](Bulk::take) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Take<T, N = [<T as IntoIterator>::Item]>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    bulk: T,
    n: <N as Pointee>::Metadata
}

impl<T, N> Take<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    pub(crate) const fn new(bulk: T, n: <N as Pointee>::Metadata) -> Take<T, N>
    {
        Self { bulk, n }
    }
}
impl<T, N> IntoIterator for Take<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    type Item = T::Item;
    type IntoIter = <<core::iter::Take<
        <T::IntoIter as ContainedIntoIter>::ContainedIntoIter
    > as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, n } = self;
        unsafe {
            bulk.into_iter()
                .contained_into_iter()
                .take(N::len_metadata(n))
                .into_contained()
                .into_iter()
        }
    }
}
impl<T, N> const Bulk for Take<T, N>
where
    T: ~const Bulk,
    N: ~const Length<Elem = T::Item> + ?Sized
{
    fn len(&self) -> usize
    {
        let Self { bulk, n } = self;
        N::len_metadata(*n).min(bulk.len())
    }
}
impl<T, A, const N: usize, const M: usize> StaticBulk for Take<T, [A; N]>
where
    T: StaticBulk<Item = A, Array = [A; M]>,
    [A; N.min(M)]:
{
    type Array = [A; N.min(M)];

    fn collect_array(self) -> Self::Array
    {
        self.into_iter().next_chunk().ok().unwrap()
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::take::<_, [_]>(0..6, 10).collect::<Vec<_>>();

        println!("{a:?}")
    }
}