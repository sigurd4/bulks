use core::{fmt, marker::{Destruct, PhantomData}};

use crate::{Bulk, DoubleEndedBulk, IntoBulk, SplitBulk, StaticBulk, util::LengthSpec};

/// Creates a bulk that yields nothing.
/// 
/// Similar to [`core::iter::empty`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// let mut nope = bulks::empty::<i32>();
///
/// let nothing = nope.collect();
/// 
/// assert_eq!(nothing, []);
/// ```
pub const fn empty<T>() -> Empty<T>
{
    Empty(PhantomData)
}

/// A bulk that yields nothing.
///
/// This `struct` is created by the [`empty()`] function. See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Empty<T>(PhantomData<T>);

impl<T> const Clone for Empty<T>
{
    fn clone(&self) -> Self
    {
        Self(PhantomData)
    }
}

impl<T> fmt::Debug for Empty<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Empty").finish()
    }
}

impl<T> IntoIterator for Empty<T>
{
    type Item = T;
    type IntoIter = core::iter::Empty<T>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::empty()
    }
}
impl<T> const IntoBulk for core::iter::Empty<T>
{
    type IntoBulk = Empty<T>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        empty()
    }
}
impl<T> const Bulk for Empty<T>
{
    fn len(&self) -> usize
    {
        0
    }
    fn is_empty(&self) -> bool
    {
        true
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        None
    }
    fn last(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        None
    }
    fn nth<L>(self, _n: L) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        L: ~const LengthSpec
    {
        None
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let _ = f;
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let _ = f;
        R::from_output(())
    }
}
impl<T> const DoubleEndedBulk for Empty<T>
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let _ = f;
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let _ = f;
        R::from_output(())
    }
}
unsafe impl<T> StaticBulk for Empty<T>
{
    type Array<U> = [U; 0];
}
impl<T, L> const SplitBulk<L> for Empty<T>
where
    L: LengthSpec
{
    type Left = Self;
    type Right = Self;

    fn split_at(self, _n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        (self.clone(), self)
    }
}

pub const trait EmptyBulk: ~const DoubleEndedBulk + StaticBulk<Array<<Self as IntoIterator>::Item> = [<Self as IntoIterator>::Item; 0]>
{

}
impl<T> const EmptyBulk for T
where
    T: ~const DoubleEndedBulk + StaticBulk<Array<<Self as IntoIterator>::Item> = [<Self as IntoIterator>::Item; 0]>
{

}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = const {
            crate::empty::<u8>().collect_array()
        };
        assert_eq!(a, [])
    }
}