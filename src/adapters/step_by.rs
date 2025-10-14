use core::ptr::Pointee;

use crate::{util::Length, Bulk, StaticBulk};

/// An iterator for stepping iterators by a custom amount.
///
/// This `struct` is created by the [`step_by`](Bulk::step_by) method on [`Bulk`]. See
/// its documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct StepBy<T, N = [<T as IntoIterator>::Item]>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    bulk: T,
    step: <N as Pointee>::Metadata
}

impl<T, N> StepBy<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    pub(crate) const fn new(bulk: T, step: <N as Pointee>::Metadata) -> StepBy<T, N>
    where
        N: ~const Length<Elem = T::Item>
    {
        assert!(N::len_metadata(step) != 0);
        Self { bulk, step }
    }
}

impl<T, N> IntoIterator for StepBy<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    type Item = T::Item;
    type IntoIter = core::iter::StepBy<T::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, step } = self;
        bulk.into_iter()
            .step_by(N::len_metadata(step))
    }
}
impl<T, N> const Bulk for StepBy<T, N>
where
    T: ~const Bulk,
    N: ~const Length<Elem = T::Item> + ?Sized
{
    fn len(&self) -> usize
    {
        let Self { bulk, step } = self;
        bulk.len()/N::len_metadata(*step)
    }
}
impl<T, A, const N: usize, const M: usize> StaticBulk for StepBy<T, [A; N]>
where
    T: StaticBulk<Item = A, Array = [A; M]>,
    [A; M/N]:
{
    type Array = [A; M/N];

    fn collect_array(self) -> Self::Array
    {
        self.into_iter().next_chunk().ok().unwrap()
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let a = a.into_bulk().step_by::<[_; 2]>(()).collect::<[_; _]>();

        println!("{a:?}")
    }
}