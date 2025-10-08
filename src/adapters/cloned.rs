use crate::{Bulk, LimitToBulk, Rev, StaticBulk};

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
    pub(crate) fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }

    pub(crate) fn into_inner(self) -> I
    {
        let Self { bulk } = self;
        bulk
    }
}

impl<'a, I, T> Default for Cloned<I>
where
    I: Bulk<Item = &'a T> + Default,
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
impl<'a, I, T> LimitToBulk for Cloned<I>
where
    I: Bulk<Item = &'a T>,
    T: Clone + 'a
{
    
}
impl<'a, I, T> Bulk for Cloned<I>
where
    I: Bulk<Item = &'a T>,
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
impl<'a, I, T, const N: usize> StaticBulk for Cloned<I>
where 
    I: StaticBulk<Item = &'a T, Array = [&'a T; N]>,
    T: Clone + 'a
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        self.into_inner().cloned_collect_array()
    }
}

pub(crate) trait StaticClonedSpec<const N: usize>: StaticBulk<Array = [<Self as IntoIterator>::Item; N]>
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