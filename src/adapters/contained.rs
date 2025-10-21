use core::usize;

use crate::util::InfiniteIterator;
use crate::util::LengthSpec;
use crate::Bulk;
use crate::DoubleEndedBulk;
use crate::IntoBulk;
use crate::StaticBulk;

pub(crate) use private::IntoContained as IntoContained;
pub(crate) use private::ContainedIntoIter as ContainedIntoIter;

pub struct Contained<I>
where
    I: Iterator
{
    iter: I
}

impl<I> Iterator for Contained<I>
where
    I: Iterator
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next()
    }
}
impl<I> ExactSizeIterator for Contained<I>
where
    I: Iterator
{
    fn len(&self) -> usize
    {
        Bulk::len(self)
    }
    fn is_empty(&self) -> bool
    {
        Bulk::is_empty(self)
    }
}
impl<I> DoubleEndedIterator for Contained<I>
where
    I: DoubleEndedIterator
{
    fn next_back(&mut self) -> Option<Self::Item>
    {
        self.iter.next_back()
    }
}

impl<I> Contained<I>
where
    I: Iterator
{
    /// # Safety
    /// 
    /// This creates a bulk that is possibly only invalid.
    /// 
    /// Always wrap this bulk in another bulk so that its length is limited.
    pub(crate) const unsafe fn new(iter: I) -> Self
    {
        Self {
            iter
        }
    }
}

impl<I> Bulk for Contained<I>
where
    I: Iterator
{
    #[inline]
    default fn len(&self) -> usize
    {
        self.iter.size_hint().1.unwrap_or(usize::MAX)
    }

    #[inline]
    default fn is_empty(&self) -> bool
    {
        Bulk::len(self) == 0
    }

    fn first(mut self) -> Option<Self::Item>
    where
        Self: Sized
    {
        self.iter.next()
    }
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        self.iter.last()
    }
    fn nth<L>(mut self, n: L) -> Option<Self::Item>
    where
        Self: Sized,
        L: LengthSpec
    {
        self.iter.nth(n.len_metadata())
    }
    
    #[inline]
    default fn for_each<F>(self, _f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        panic!("Possibly infinite iterator.")
    }
    
    #[inline]
    default fn try_for_each<F, R>(self, _f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        panic!("Possibly infinite iterator.")
    }
}
impl<I> Bulk for Contained<I>
where
    I: ExactSizeIterator
{
    #[inline]
    fn len(&self) -> usize
    {
        self.iter.len()
    }

    #[inline]
    fn is_empty(&self) -> bool
    {
        self.iter.is_empty()
    }
    
    #[inline]
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.iter.for_each(f);
    }
    
    #[inline]
    fn try_for_each<F, R>(mut self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        self.iter.try_for_each(f)
    }
}
impl<I> DoubleEndedBulk for Contained<I>
where
    I: DoubleEndedIterator
{
    #[inline]
    default fn rev_for_each<F>(self, _f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        panic!("Possibly infinite iterator.")
    }
    
    #[inline]
    default fn try_rev_for_each<F, R>(self, _f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        panic!("Possibly infinite iterator.")
    }
}
impl<I> DoubleEndedBulk for Contained<I>
where
    I: DoubleEndedIterator + ExactSizeIterator
{
    #[inline]
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.iter.rev().for_each(f);
    }
    
    #[inline]
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        self.iter.rev().try_for_each(f)
    }
}
unsafe impl<I> StaticBulk for Contained<I>
where
    I: InfiniteIterator
{
    type Array<U> = [U; usize::MAX];
}

mod private
{
    use crate::{util::Same, Contained, IntoBulk};

    /// # Safety
    /// 
    /// This creates a bulk that is possibly only invalid.
    /// 
    /// Always wrap this bulk in another bulk so that its length is limited.
    #[const_trait]
    pub unsafe trait IntoContained: IntoIterator
    {
        type IntoContained: ~const IntoBulk<Item = Self::Item, IntoIter: ExactSizeIterator<Item = Self::Item>>;

        /// # Safety
        /// 
        /// This creates a bulk that is possibly only invalid.
        /// 
        /// Always wrap this bulk in another bulk so that its length is limited.
        unsafe fn into_contained(self) -> Self::IntoContained;
    }
    unsafe impl<T> IntoContained for T
    where
        T: IntoIterator
    {
        default type IntoContained = Contained<T::IntoIter>;

        default unsafe fn into_contained(self) -> Self::IntoContained
        {
            unsafe {
                Contained::new(self.into_iter()).same().ok().unwrap()
            }
        }
    }
    unsafe impl<T> const IntoContained for T
    where
        T: ~const IntoBulk
    {
        type IntoContained = T;

        unsafe fn into_contained(self) -> Self::IntoContained
        {
            self
        }
    }

    pub unsafe trait ContainedIntoIter: IntoIterator<IntoIter: ExactSizeIterator>
    {
        type ContainedIntoIter: Iterator<Item = Self::Item>;

        unsafe fn contained_into_iter(self) -> Self::ContainedIntoIter;
    }
    unsafe impl<T> ContainedIntoIter for T
    where
        T: IntoIterator<IntoIter: ExactSizeIterator>
    {
        default type ContainedIntoIter = T::IntoIter;

        default unsafe fn contained_into_iter(self) -> Self::ContainedIntoIter
        {
            self.into_iter().same().ok().unwrap()
        }
    }
    unsafe impl<I> ContainedIntoIter for Contained<I>
    where
        I: Iterator
    {
        type ContainedIntoIter = I;

        unsafe fn contained_into_iter(self) -> Self::ContainedIntoIter
        {
            self.iter
        }
    }

    pub trait EitherIntoBulk<B>: IntoIterator
    where
        B: IntoIterator
    {
        type EitherIntoBulk: IntoIterator;
    }
    impl<T, B> EitherIntoBulk<B> for T
    where
        T: IntoIterator,
        B: IntoIterator
    {
        default type EitherIntoBulk = T;
    }
    impl<T, B> EitherIntoBulk<B> for T
    where
        T: IntoIterator,
        B: IntoBulk
    {
        type EitherIntoBulk = B;
    }
}

#[rustc_on_unimplemented(
    message = "value of type `{Self}` cannot be zipped with `{B}` in bulk",
    label = "neither `{Self}` nor `{B}` can be converted into bulks",
)]
pub trait EitherIntoBulk<B>: private::EitherIntoBulk<B, EitherIntoBulk: IntoBulk>
where
    B: IntoIterator
{
    
}
impl<T, B> EitherIntoBulk<B> for T
where
    T: private::EitherIntoBulk<B, EitherIntoBulk: IntoBulk>,
    B: IntoIterator
{
    
}

#[rustc_on_unimplemented(
    message = "value of type `{Self}` cannot be zipped with `{B}` in bulk",
    label = "value of type `{Self}` cannot be zipped with `{B}` in bulk",
)]
pub const trait IntoContainedBy<B>: ~const IntoContained + EitherIntoBulk<B> + Sized
where
    B: IntoIterator
{

}
impl<T, B> const IntoContainedBy<B> for T
where
    T: ~const IntoContained + EitherIntoBulk<B>,
    B: IntoIterator
{

}