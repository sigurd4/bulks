use core::marker::Destruct;

use crate::{Bulk, DoubleEndedBulk, StaticBulk};


/// A double-ended bulk with the direction inverted.
///
/// This `struct` is created by the [`rev`](Bulk::rev) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Rev<I>
where
    I: DoubleEndedBulk
{
    bulk: I,
}

impl<I> Rev<I>
where
    I: DoubleEndedBulk
{
    pub(crate) const fn new(bulk: I) -> Self
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
    /// let s = b"foobar";
    /// let mut s2 = s.bulk()
    ///     .copied()
    ///     .rev()
    ///     .into_inner()
    ///     .collect::<[_; _]>();
    /// assert_eq!(&s2, b"foobar");
    /// ```
    pub const fn into_inner(self) -> I
    {
        let Self { bulk } = self;
        bulk
    }
}

impl<I> const Default for Rev<I>
where
    I: ~const Bulk + DoubleEndedBulk + ~const Default
{
    fn default() -> Self
    {
        I::default().rev()
    }
}

impl<I> IntoIterator for Rev<I>
where
    I: DoubleEndedBulk
{
    type IntoIter = core::iter::Rev<I::IntoIter>;
    type Item = I::Item;

    fn into_iter(self) -> Self::IntoIter
    {
        self.into_inner().into_iter().rev()
    }
}
impl<I> const Bulk for Rev<I>
where
    I: ~const Bulk + ~const DoubleEndedBulk
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
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk } = self;
        bulk.rev_for_each(f);
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk } = self;
        bulk.try_rev_for_each(f)
    }
}
impl<I> const DoubleEndedBulk for Rev<I>
where
    I: ~const Bulk + DoubleEndedBulk
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk } = self;
        bulk.for_each(f);
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk } = self;
        bulk.try_for_each(f)
    }
}
unsafe impl<I> StaticBulk for Rev<I>
where
    I: StaticBulk + DoubleEndedBulk
{
    type Array<U> = I::Array<U>;
}