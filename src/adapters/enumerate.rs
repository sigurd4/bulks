use crate::{Bulk, StaticBulk};

/// A bulk that yields the element's index and the element.
///
/// This `struct` is created by the [`enumerate`](Bulk::enumerate) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Enumerate<I>
where
    I: Bulk
{
    bulk: I,
}

impl<I, T> Enumerate<I>
where
    I: Bulk<Item = T>
{
    pub(crate) const fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }

    pub(crate) const fn into_inner(self) -> I
    {
        crate::const_inner!(
            for<{I, T}> Enumerate{ bulk } in self => Enumerate<I> => I
            where {
                I: Bulk<Item = T>
            }
            {
                bulk
            }
        )
    }
}

impl<I, T> const Default for Enumerate<I>
where
    I: ~const Bulk<Item = T> + ~const Default
{
    fn default() -> Self
    {
        I::default().enumerate()
    }
}

impl<I, T> IntoIterator for Enumerate<I>
where
    I: Bulk<Item = T>
{
    type IntoIter = core::iter::Enumerate<I::IntoIter>;
    type Item = (usize, T);

    fn into_iter(self) -> Self::IntoIter
    {
        self.into_inner().into_iter().enumerate()
    }
}
impl<I, T> const Bulk for Enumerate<I>
where
    I: ~const Bulk<Item = T>
{
    fn len(&self) -> usize
    {
        let Self { bulk } = self;
        bulk.len()
    }

    fn is_empty(&self) -> bool
    {
        let Self { bulk } = self;
        bulk.is_empty()
    }
}
impl<I, T, const N: usize> StaticBulk for Enumerate<I>
where 
    I: StaticBulk<Item = T, Array = [T; N]>
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        self.into_iter().next_chunk().ok().unwrap()
    }
}