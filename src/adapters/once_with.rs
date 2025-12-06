use core::{fmt, marker::Destruct};

use array_trait::length::LengthValue;

use crate::{Bulk, DoubleEndedBulk, Once, RepeatN, RepeatNWith, SplitBulk, util::TakeOne};

/// Creates a bulk that lazily generates a value exactly once by invoking
/// the provided closure.
///
/// Unlike [`once()`](crate::once), this function will lazily generate the value on request.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// // one is the loneliest number
/// let mut one: [_; _] = bulks::once_with(|| 1).collect();
///
/// // just one, that's all we get
/// assert_eq!(one, [1])
/// ```
pub const fn once_with<A, F>(value: F) -> OnceWith<F>
where
    F: FnOnce() -> A
{
    OnceWith(value)
}

/// A bulk that yields a single element of type `A` by
/// applying the provided closure `F: FnOnce() -> A`.
///
/// This `struct` is created by the [`once_with()`] function.
/// See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct OnceWith<F>(F)
where
    F: FnOnce<()>;

impl<F, A> fmt::Debug for OnceWith<F>
where
    F: FnOnce() -> A
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.write_str("OnceWith")
    }
}

impl<F, A> IntoIterator for OnceWith<F>
where
    F: FnOnce() -> A
{
    type Item = A;
    type IntoIter = core::iter::OnceWith<F>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::once_with(self.0)
    }
}
impl<F, A> const Bulk for OnceWith<F>
where
    F: ~const FnOnce() -> A
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
        Some(self.0())
    }
    fn last(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        Some(self.0())
    }

    fn for_each<FF>(self, mut f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        f(self.0())
    }
    fn try_for_each<FF, R>(self, mut f: FF) -> R
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = ()>
    {
        f(self.0())
    }
}
impl<F, A> const DoubleEndedBulk for OnceWith<F>
where
    F: ~const FnOnce() -> A
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
impl<F, A, L> const SplitBulk<L> for OnceWith<F>
where
    F: ~const FnOnce() -> A,
    L: LengthValue,
    RepeatNWith<TakeOne<F>, [(); 1]>: ~const SplitBulk<L, Item = A>
{
    type Left = <RepeatNWith<TakeOne<F>, [(); 1]> as SplitBulk<L>>::Left;
    type Right = <RepeatNWith<TakeOne<F>, [(); 1]> as SplitBulk<L>>::Right;

    fn split_at(bulk: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        RepeatNWith::from(bulk).split_at(n)
    }
}

impl<F, A> const From<OnceWith<F>> for Once<A>
where
    F: ~const FnOnce() -> A
{
    fn from(value: OnceWith<F>) -> Self
    {
        crate::once(value.0())
    }
}
impl<F, A> const From<OnceWith<F>> for RepeatN<A, [(); 1]>
where
    F: ~const FnOnce() -> A,
    A: Clone
{
    fn from(value: OnceWith<F>) -> Self
    {
        crate::repeat_n((value.0)(), [(); 1])
    }
}
impl<F, A> const From<OnceWith<F>> for RepeatNWith<F, [(); 1]>
where
    F: FnMut() -> A
{
    fn from(value: OnceWith<F>) -> Self
    {
        crate::repeat_n_with(value.0, [(); 1])
    }
}
impl<F, A> const From<OnceWith<F>> for RepeatNWith<TakeOne<F>, [(); 1]>
where
    F: FnOnce() -> A
{
    fn from(value: OnceWith<F>) -> Self
    {
        crate::repeat_n_with(TakeOne::new(value.0), [(); 1])
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        const fn one() -> i32
        {
            1
        }

        let a = const {
            crate::once_with(one).collect::<[_; _], _>()
        };
        assert_eq!(a, [1])
    }
}