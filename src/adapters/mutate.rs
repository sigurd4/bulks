use core::{fmt, marker::Destruct};

use crate::{util::Mutator, Bulk, StaticBulk};

/// A bulk that calls a function with a mutable reference to each element before
/// yielding it.
///
/// This `struct` is created by the [`mutate`](Bulk::mutate) method on [`Bulk`]. See its
/// documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Mutate<I, F>
where
    I: Bulk,
    F: FnMut(&mut I::Item)
{
    bulk: I,
    f: F
}

impl<I, F> Mutate<I, F>
where
    I: Bulk,
    F: FnMut(&mut I::Item)
{
    pub(crate) const fn new(bulk: I, f: F) -> Self
    {
        Self {
            bulk,
            f
        }
    }
}

impl<I, F> fmt::Debug for Mutate<I, F>
where
    I: Bulk + fmt::Debug,
    F: FnMut(&mut I::Item)
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let Self { bulk, f: _ } = self;
        f.debug_struct("Inspect").field("bulk", bulk).finish()
    }
}

impl<I, F> IntoIterator for Mutate<I, F>
where
    I: Bulk,
    F: FnMut(&mut I::Item)
{
    type Item = I::Item;
    type IntoIter = core::iter::Map<I::IntoIter, Mutator<F>>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter().map(Mutator(f))
    }
}
impl<I, F> const Bulk for Mutate<I, F>
where
    I: ~const Bulk,
    F: FnMut(&mut I::Item)
{
    fn len(&self) -> usize
    {
        self.bulk.len()
    }

    fn is_empty(&self) -> bool
    {
        self.bulk.is_empty()
    }
}

impl<I, F, T, const N: usize> const StaticBulk for Mutate<I, F>
where
    I: ~const StaticBulk<Item = T, Array = [T; N]> + ~const StaticMutateSpec<F>,
    F: ~const FnMut(&mut T) + ~const Destruct
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        I::mutate_collect_array(self)
    }
}

const trait StaticMutateSpec<F>: ~const StaticBulk + Sized
where
    F: FnMut(&mut <Self as IntoIterator>::Item)
{
    fn mutate_collect_array(bulk: Mutate<Self, F>) -> Self::Array;
}
impl<I, F, T, const N: usize> StaticMutateSpec<F> for I
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    F: FnMut(&mut T)
{
    default fn mutate_collect_array(bulk: Mutate<Self, F>) -> [T; N]
    {
        let Mutate { bulk, mut f } = bulk;
        let mut array = bulk.collect_array();
        for x in &mut array
        {
            f(x)
        }
        array
    }
}
impl<I, F, T, const N: usize> const StaticMutateSpec<F> for I
where
    I: ~const StaticBulk<Item = T, Array = [T; N]>,
    F: ~const FnMut(&mut T) + ~const Destruct
{
    fn mutate_collect_array(bulk_inspect: Mutate<Self, F>) -> [T; N]
    {
        let Mutate { bulk, f } = &bulk_inspect;
        let bulk = unsafe {core::ptr::read(bulk)};
        let mut f = unsafe {core::ptr::read(f)};
        core::mem::forget(bulk_inspect);
        
        let mut array = bulk.collect_array();
        let mut i = 0;
        while i < N
        {
            f(&mut array[i]);
            i += 1
        }
        array
    }
}