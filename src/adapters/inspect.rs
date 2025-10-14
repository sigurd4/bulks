use core::{fmt, marker::Destruct};

use crate::{Bulk, StaticBulk};

/// A bulk that calls a function with a reference to each element before
/// yielding it.
///
/// This `struct` is created by the [`inspect`](Bulk::inspect) method on [`Bulk`]. See its
/// documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Inspect<I, F>
where
    I: Bulk,
    F: FnMut(&I::Item)
{
    bulk: I,
    f: F
}

impl<I, F> Inspect<I, F>
where
    I: Bulk,
    F: FnMut(&I::Item)
{
    pub(crate) const fn new(bulk: I, f: F) -> Self
    {
        Self {
            bulk,
            f
        }
    }
}

impl<I, F> fmt::Debug for Inspect<I, F>
where
    I: Bulk + fmt::Debug,
    F: FnMut(&I::Item)
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let Self { bulk, f: _ } = self;
        f.debug_struct("Inspect").field("bulk", bulk).finish()
    }
}

impl<I, F> IntoIterator for Inspect<I, F>
where
    I: Bulk,
    F: FnMut(&I::Item)
{
    type Item = I::Item;
    type IntoIter = core::iter::Inspect<I::IntoIter, F>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter().inspect(f)
    }
}
impl<I, F> const Bulk for Inspect<I, F>
where
    I: ~const Bulk,
    F: FnMut(&I::Item)
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

impl<I, F, T, const N: usize> const StaticBulk for Inspect<I, F>
where
    I: ~const StaticBulk<Item = T, Array = [T; N]> + ~const StaticInspectSpec<F>,
    F: ~const FnMut(&T) + ~const Destruct
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        I::inspect_collect_array(self)
    }
}

const trait StaticInspectSpec<F>: ~const StaticBulk + Sized
where
    F: FnMut(&<Self as IntoIterator>::Item)
{
    fn inspect_collect_array(bulk: Inspect<Self, F>) -> Self::Array;
}
impl<I, F, T, const N: usize> StaticInspectSpec<F> for I
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    F: FnMut(&T)
{
    default fn inspect_collect_array(bulk: Inspect<Self, F>) -> [T; N]
    {
        let Inspect { bulk, mut f } = bulk;
        let array = bulk.collect_array();
        for x in &array
        {
            f(x)
        }
        array
    }
}
impl<I, F, T, const N: usize> const StaticInspectSpec<F> for I
where
    I: ~const StaticBulk<Item = T, Array = [T; N]>,
    F: ~const FnMut(&T) + ~const Destruct
{
    fn inspect_collect_array(bulk_inspect: Inspect<Self, F>) -> [T; N]
    {
        let Inspect { bulk, f } = &bulk_inspect;
        let bulk = unsafe {core::ptr::read(bulk)};
        let mut f = unsafe {core::ptr::read(f)};
        core::mem::forget(bulk_inspect);
        
        let array = bulk.collect_array();
        let mut i = 0;
        while i < N
        {
            f(&array[i]);
            i += 1
        }
        array
    }
}