use crate::{Bulk, Rev, StaticBulk};

/// A bulk that clones the elements of an underlying bulk.
///
/// This `struct` is created by the [`cloned`](Bulk::cloned) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Cloned<I>
where
    I: Bulk,
    core::iter::Cloned<I::IntoIter>: Iterator<Item: Clone>
{
    bulk: I,
}

impl<'a, I, T> Cloned<I>
where
    I: Bulk<Item = &'a T>,
    T: Clone + 'a
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
            for<{'a, I, T}> Cloned{ bulk } in self => Cloned<I> => I
            where {
                I: Bulk<Item = &'a T>,
                T: Clone + 'a
            }
            {
                bulk
            }
        )
    }
}

impl<'a, I, T> const Default for Cloned<I>
where
    I: ~const Bulk<Item = &'a T> + ~const Default,
    T: Clone + 'a
{
    fn default() -> Self
    {
        I::default().cloned()
    }
}

impl<'a, I, T> IntoIterator for Cloned<I>
where
    I: Bulk<Item = &'a T>,
    T: Clone + 'a
{
    type IntoIter = core::iter::Cloned<I::IntoIter>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        self.into_inner().into_iter().cloned()
    }
}
impl<'a, I, T> const Bulk for Cloned<I>
where
    I: ~const Bulk<Item = &'a T>,
    T: Clone + 'a
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
impl<'a, I, T, const N: usize> const StaticBulk for Cloned<I>
where 
    I: ~const StaticClonedSpec<N, Item = &'a T, Array = [&'a T; N]>,
    T: Clone + 'a
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        self.into_inner().cloned_collect_array()
    }
}

pub(crate) const trait StaticClonedSpec<const N: usize>: ~const StaticBulk<Array = [<Self as IntoIterator>::Item; N]>
where
    core::iter::Cloned<Self::IntoIter>: Iterator<Item: Clone>
{
    fn cloned_collect_array(self) -> [<core::iter::Cloned<Self::IntoIter> as Iterator>::Item; N];
}

impl<'a, I, T, const N: usize> StaticClonedSpec<N> for I
where
    I: StaticBulk<Item = &'a T, Array = [&'a T; N]>,
    T: Clone + 'a
{
    default fn cloned_collect_array(self) -> [T; N]
    {
        self.into_iter().cloned().next_chunk().ok().unwrap()
    }
}

impl<'a, I, T, const N: usize> StaticClonedSpec<N> for Rev<I>
where
    I: StaticBulk<Item = &'a T, Array = [&'a T; N], IntoIter: DoubleEndedIterator>,
    T: Copy + 'a
{
    fn cloned_collect_array(self) -> [T; N]
    {
        self.into_inner().copied().collect_array()
    }
}