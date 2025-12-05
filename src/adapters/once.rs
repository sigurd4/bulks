use core::{borrow::Borrow, marker::Destruct};

use array_trait::length::LengthValue;

use crate::{Bulk, DoubleEndedBulk, OnceWith, RandomAccessBulk, RepeatN, RepeatNWith, SplitBulk, StaticBulk, util::{FlatRef, TakeOne, YieldOnce}};

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
    Once(value)
}

/// A bulk that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function. See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Once<T>(T);

impl<T> IntoIterator for Once<T>
{
    type Item = T;
    type IntoIter = core::iter::Once<T>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::once(self.0)
    }
}
impl<T> const Bulk for Once<T>
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
impl<T> const DoubleEndedBulk for Once<T>
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
impl<A, L> const SplitBulk<L> for Once<A>
where
    L: LengthValue,
    OnceWith<YieldOnce<A>>: ~const SplitBulk<L, Item = A, Left: ~const Bulk, Right: ~const Bulk>
{
    type Left = <OnceWith<YieldOnce<A>> as SplitBulk<L>>::Left;
    type Right = <OnceWith<YieldOnce<A>> as SplitBulk<L>>::Right;

    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        OnceWith::from(self).split_at(n)
    }
}
impl<'a, T, R> const RandomAccessBulk<'a> for Once<T>
where
    Self: 'a,
    T: FlatRef<'a, FlatRef = R>,
    &'a T: ~const Borrow<R>,
    R: FlatRef<'a, FlatRef = R> + ~const Destruct + Copy + 'a,
{
    type ItemRef = R;
    type EachRef = Once<R>;

    fn each_ref(Self(value): &'a Self) -> Self::EachRef
    {
        crate::once(*(&value).borrow())
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
impl<A> const From<Once<A>> for OnceWith<YieldOnce<A>>
{
    fn from(value: Once<A>) -> Self
    {
        crate::once_with(YieldOnce::new(value.0))
    }
}
impl<A> const From<Once<A>> for RepeatN<A, [(); 1]>
where
    A: Clone
{
    fn from(value: Once<A>) -> Self
    {
        crate::repeat_n(value.0, [(); 1])
    }
}
impl<A> const From<Once<A>> for RepeatNWith<TakeOne<YieldOnce<A>>, [(); 1]>
{
    fn from(value: Once<A>) -> Self
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