use crate::{Bulk, StaticBulk};


/// A double-ended bulk with the direction inverted.
///
/// This `struct` is created by the [`rev`](Bulk::rev) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Rev<I>
where
    I: Bulk<IntoIter: DoubleEndedIterator>
{
    bulk: I,
}

impl<I> Rev<I>
where
    I: Bulk<IntoIter: DoubleEndedIterator>
{
    pub(crate) fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }

    /// Consumes the `Rev`, returning the inner bulk.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bulks::*;
    ///
    /// let s = "foobar";
    /// let mut s2 = s.bulk().rev().into_inner().collect();
    /// assert_eq!(s2, "foobar");
    /// ```
    pub fn into_inner(self) -> I
    {
        let Self { bulk } = self;
        bulk
    }
}

impl<I> Default for Rev<I>
where
    I: Bulk<IntoIter: DoubleEndedIterator> + Default
{
    fn default() -> Self
    {
        I::default().rev()
    }
}

impl<I> IntoIterator for Rev<I>
where
    I: Bulk<IntoIter: DoubleEndedIterator>
{
    type IntoIter = core::iter::Rev<I::IntoIter>;
    type Item = I::Item;

    fn into_iter(self) -> Self::IntoIter
    {
        self.into_inner().into_iter().rev()
    }
}
impl<I> Bulk for Rev<I>
where
    I: Bulk<IntoIter: DoubleEndedIterator>
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
impl<I, T, const N: usize> StaticBulk for Rev<I>
where
    I: StaticBulk<Item = T, Array = [T; N], IntoIter: DoubleEndedIterator>
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        self.into_inner().rev_collect_array()
    }
}

pub(crate) trait StaticRevSpec<const N: usize>: StaticBulk<Array = [<Self as IntoIterator>::Item; N], IntoIter: DoubleEndedIterator>
{
    fn rev_collect_array(self) -> [<Self as IntoIterator>::Item; N];
}

impl<T, I, const N: usize> StaticRevSpec<N> for I
where
    I: StaticBulk<Item = T, Array = [T; N], IntoIter: DoubleEndedIterator>
{
    default fn rev_collect_array(self) -> [<Self as IntoIterator>::Item; N]
    {
        self.into_iter().rev().next_chunk().ok().unwrap()
    }
}

impl<T, I, const N: usize> StaticRevSpec<N> for Rev<I>
where
    I: StaticBulk<Item = T, Array = [T; N], IntoIter: DoubleEndedIterator>
{
    fn rev_collect_array(self) -> [<Self as IntoIterator>::Item; N]
    {
        self.into_inner().collect_array()
    }
}