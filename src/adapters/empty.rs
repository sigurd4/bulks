use core::{borrow::{Borrow, BorrowMut}, fmt, marker::{Destruct, PhantomData}};

use array_trait::length::LengthValue;

use crate::{Bulk, DoubleEndedBulk, IntoBulk, RandomAccessBulk, InplaceBulk, SplitBulk, StaticBulk};

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
/// let nothing: [_; _] = nope.collect();
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
pub struct Empty<T, P = T>(PhantomData<(T, P)>)
where
    T: Borrow<P>;

impl<T, P> const Clone for Empty<T, P>
where
    T: Borrow<P>
{
    fn clone(&self) -> Self
    {
        Self(PhantomData)
    }
}

impl<T, P> fmt::Debug for Empty<T, P>
where
    T: Borrow<P>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Empty").finish()
    }
}

impl<T, P> IntoIterator for Empty<T, P>
where
    T: Borrow<P>
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
impl<T, P> const Bulk for Empty<T, P>
where
    T: Borrow<P>
{
    type MinLength = [(); 0];
    type MaxLength = [(); 0];

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
        L: LengthValue
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
impl<T, P> const DoubleEndedBulk for Empty<T, P>
where
    T: Borrow<P>
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
impl<T, P, L> const SplitBulk<L> for Empty<T, P>
where
    L: LengthValue,
    T: Borrow<P>
{
    type Left = Self;
    type Right = Self;

    fn split_at(bulk: Self, _n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        (bulk.clone(), bulk)
    }
}
impl<T, P> const RandomAccessBulk for Empty<T, P>
where
    T: Borrow<P>
{
    type ItemPointee = P;
    type EachRef<'a> = Empty<&'a P, P>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(Self(PhantomData): &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        Empty(PhantomData)
    }
}
impl<T, P> const InplaceBulk for Empty<T, P>
where
    T: BorrowMut<P>
{
    type EachMut<'a> = Empty<&'a mut P, P>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_mut<'a>(Self(PhantomData): &'a mut Self) -> Self::EachMut<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        Empty(PhantomData)
    }
}

pub const trait EmptyBulk: ~const DoubleEndedBulk + StaticBulk<Length = [(); 0]>
{

}
impl<T> const EmptyBulk for T
where
    T: ~const DoubleEndedBulk + StaticBulk<Length = [(); 0]>
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