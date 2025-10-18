use core::ptr::Pointee;

use crate::{util::Length, Bulk, StaticBulk};

/// A bulk that skips over `n` elements of `bulk`.
///
/// This `struct` is created by the [`skip`](Bulk::skip) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Skip<T, N = [<T as IntoIterator>::Item]>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    bulk: T,
    n: <N as Pointee>::Metadata
}

impl<T, N> Skip<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    pub(crate) const fn new(bulk: T, n: <N as Pointee>::Metadata) -> Skip<T, N>
    {
        Self { bulk, n }
    }
}
impl<T, N> IntoIterator for Skip<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    type Item = T::Item;
    type IntoIter = core::iter::Skip<T::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, n } = self;
        bulk.into_iter()
            .skip(N::len_metadata(n))
    }
}
impl<T, N> const Bulk for Skip<T, N>
where
    T: ~const Bulk,
    N: ~const Length<Elem = T::Item> + ?Sized
{
    fn len(&self) -> usize
    {
        let Self { bulk, n } = self;
        bulk.len().saturating_sub(N::len_metadata(*n))
    }

    fn is_empty(&self) -> bool
    {
        let Self { bulk, n } = self;
        bulk.len() > N::len_metadata(*n)
    }
}
impl<T, A, const N: usize, const M: usize> StaticBulk for Skip<T, [A; N]>
where
    T: StaticBulk<Item = A, Array = [A; M]>,
    [A; M.saturating_sub(N)]:
{
    type Array = [A; M.saturating_sub(N)];

    fn collect_array(self) -> Self::Array
    {
        self.into_iter().next_chunk().ok().unwrap()
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3, 4, 5];
        let b = a.into_bulk().skip::<[_; 2]>(()).collect::<[_; _]>();

        println!("{b:?}")
    }
}