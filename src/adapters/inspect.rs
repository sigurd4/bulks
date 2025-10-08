use core::fmt;

use crate::{Bulk, LimitToBulk, StaticBulk};

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
    pub(crate) fn new(bulk: I, f: F) -> Self
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
impl<I, F> LimitToBulk for Inspect<I, F>
where
    I: Bulk,
    F: FnMut(&I::Item)
{
    
}
impl<I, F> Bulk for Inspect<I, F>
where
    I: Bulk,
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
impl<I, F, T, const N: usize> StaticBulk for Inspect<I, F>
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    F: FnMut(&T)
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        let Self { bulk, mut f } = self;
        let array = bulk.collect_array();
        for x in &array
        {
            f(x)
        }
        array
    }
}