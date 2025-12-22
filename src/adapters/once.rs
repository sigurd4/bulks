use core::{borrow::{Borrow, BorrowMut}, marker::{Destruct, PhantomData}};

use array_trait::length::LengthValue;

use crate::{Bulk, DoubleEndedBulk, InplaceBulk, OnceWith, RandomAccessBulk, RepeatN, RepeatNWith, SplitBulk, StaticBulk, util::{TakeOne, YieldOnce}};

/// Creates a bulk that yields an element exactly once.
/// 
/// Similar to [`core::iter::once`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// // one is the loneliest number
/// let mut one: [_; _] = bulks::once(1).collect();
///
/// // just one, that's all we get
/// assert_eq!(one, [1])
/// ```
pub const fn once<T>(value: T) -> Once<T>
{
    Once(value, PhantomData)
}

/// A bulk that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function. See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Once<T, P = T>(T, PhantomData<P>)
where
    T: Borrow<P>;

impl<T, P> IntoIterator for Once<T, P>
where
    T: Borrow<P>
{
    type Item = T;
    type IntoIter = core::iter::Once<T>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::once(self.0)
    }
}
impl<T, P> const Bulk for Once<T, P>
where
    T: Borrow<P>
{
    type MinLength = [(); 1];
    type MaxLength = [(); 1];

    fn len(&self) -> usize
    {
        1
    }
    fn is_empty(&self) -> bool
    {
        false
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        Some(self.0)
    }
    fn last(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        Some(self.0)
    }

    fn for_each<FF>(self, mut f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        f(self.0)
    }
    fn try_for_each<FF, R>(self, mut f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        f(self.0)
    }
}
impl<T, P> const DoubleEndedBulk for Once<T, P>
where
    T: Borrow<P>
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        self.for_each(f);
    }
    fn try_rev_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        self.try_for_each(f)
    }
}
impl<T, L, P> const SplitBulk<L> for Once<T, P>
where
    T: Borrow<P>,
    L: LengthValue,
    OnceWith<YieldOnce<T>>: ~const SplitBulk<L, Item = T, Left: ~const Bulk, Right: ~const Bulk>
{
    type Left = <OnceWith<YieldOnce<T>> as SplitBulk<L>>::Left;
    type Right = <OnceWith<YieldOnce<T>> as SplitBulk<L>>::Right;

    fn split_at(bulk: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        OnceWith::from(bulk).split_at(n)
    }
}
impl<T, P> const RandomAccessBulk for Once<T, P>
where
    T: ~const Borrow<P>
{
    type ItemPointee = P;
    type EachRef<'a> = Once<&'a P, P>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(Self(value, PhantomData): &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a
    {
        Once(value.borrow(), PhantomData)
    }
}
impl<T, P> const InplaceBulk for Once<T, P>
where
    T: ~const BorrowMut<P>
{
    type EachMut<'a> = Once<&'a mut P, P>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_mut<'a>(Self(value, PhantomData): &'a mut Self) -> Self::EachMut<'a>
    where
        Self::ItemPointee: 'a
    {
        Once(value.borrow_mut(), PhantomData)
    }
}

pub const trait OnceBulk: ~const DoubleEndedBulk + StaticBulk<Array<<Self as IntoIterator>::Item> = [<Self as IntoIterator>::Item; 1]>
{

}
impl<T> const OnceBulk for T
where
    T: ~const DoubleEndedBulk + StaticBulk<Array<<Self as IntoIterator>::Item> = [<Self as IntoIterator>::Item; 1]>
{

}
impl<A, R> const From<Once<A, R>> for OnceWith<YieldOnce<A>>
where
    A: Borrow<R>
{
    fn from(value: Once<A, R>) -> Self
    {
        crate::once_with(YieldOnce::new(value.0))
    }
}
impl<A, R> const From<Once<A, R>> for RepeatN<A, [(); 1]>
where
    A: Clone + Borrow<R>
{
    fn from(value: Once<A, R>) -> Self
    {
        crate::repeat_n(value.0, [(); 1])
    }
}
impl<A, R> const From<Once<A, R>> for RepeatNWith<TakeOne<YieldOnce<A>>, [(); 1]>
where
    A: Borrow<R>
{
    fn from(value: Once<A, R>) -> Self
    {
        crate::repeat_n_with(TakeOne::new(YieldOnce::new(value.0)), [(); 1])
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = const {
            crate::once(1).collect::<[_; _], _>()
        };
        assert_eq!(a, [1])
    }
}