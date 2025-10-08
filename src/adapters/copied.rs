use crate::{Bulk, LimitToBulk, Rev, StaticBulk};

/// A bulk that copies the elements of an underlying bulk.
///
/// This `struct` is created by the [`copied`](Bulk::copied) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Copied<I>
where
    I: Bulk,
    core::iter::Copied<I::IntoIter>: Iterator<Item: Copy>
{
    bulk: I,
}

impl<'a, I, T> Copied<I>
where
    I: Bulk<Item = &'a T>,
    T: Copy + 'a
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

impl<'a, I, T> Default for Copied<I>
where
    I: Bulk<Item = &'a T> + Default,
    T: Copy + 'a
{
    fn default() -> Self
    {
        I::default().copied()
    }
}

impl<'a, I, T> IntoIterator for Copied<I>
where
    I: Bulk<Item = &'a T>,
    T: Copy + 'a
{
    type IntoIter = core::iter::Copied<I::IntoIter>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        self.into_inner().into_iter().copied()
    }
}
impl<'a, I, T> LimitToBulk for Copied<I>
where
    I: Bulk<Item = &'a T>,
    T: Copy + 'a
{
    
}
impl<'a, I, T> Bulk for Copied<I>
where
    I: Bulk<Item = &'a T>,
    T: Copy + 'a
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
impl<'a, I, T, const N: usize> StaticBulk for Copied<I>
where 
    I: StaticBulk<Item = &'a T, Array = [&'a T; N]>,
    T: Copy + 'a
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        self.into_inner().copied_collect_array()
    }
}

pub(crate) trait StaticCopiedSpec<const N: usize>: StaticBulk<Array = [<Self as IntoIterator>::Item; N]>
where
    core::iter::Copied<Self::IntoIter>: Iterator<Item: Copy>
{
    fn copied_collect_array(self) -> [<core::iter::Copied<Self::IntoIter> as Iterator>::Item; N];
}

impl<'a, I, T, const N: usize> StaticCopiedSpec<N> for I
where
    I: StaticBulk<Item = &'a T, Array = [&'a T; N]>,
    T: Copy + 'a
{
    default fn copied_collect_array(self) -> [T; N]
    {
        self.into_iter().copied().next_chunk().ok().unwrap()
    }
}

impl<'a, I, T, const N: usize> StaticCopiedSpec<N> for Rev<I>
where
    I: StaticBulk<Item = &'a T, Array = [&'a T; N], IntoIter: DoubleEndedIterator>,
    T: Copy + 'a
{
    fn copied_collect_array(self) -> [T; N]
    {
        self.into_inner().copied().rev().collect()
    }
}