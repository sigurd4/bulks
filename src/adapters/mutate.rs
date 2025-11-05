use core::{fmt, marker::Destruct, ops::Try};

use crate::{Bulk, DoubleEndedBulk, SplitBulk, StaticBulk, util::{LengthSpec, Mutator}};

/// A bulk that calls a function with a mutable reference to each element before
/// yielding it.
///
/// This `struct` is created by the [`mutate`](Bulk::mutate) method on [`Bulk`]. See its
/// documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Mutate<I, F>
where
    I: Bulk,
    F: FnMut(&mut I::Item)
{
    bulk: I,
    f: F
}

impl<I, F> Mutate<I, F>
where
    I: Bulk,
    F: FnMut(&mut I::Item)
{
    pub(crate) const fn new(bulk: I, f: F) -> Self
    {
        Self {
            bulk,
            f
        }
    }
}

impl<I, F> fmt::Debug for Mutate<I, F>
where
    I: Bulk + fmt::Debug,
    F: FnMut(&mut I::Item)
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let Self { bulk, f: _ } = self;
        f.debug_struct("Inspect").field("bulk", bulk).finish()
    }
}

impl<I, F> IntoIterator for Mutate<I, F>
where
    I: Bulk,
    F: FnMut(&mut I::Item)
{
    type Item = I::Item;
    type IntoIter = core::iter::Map<I::IntoIter, Mutator<F>>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter().map(Mutator(f))
    }
}
impl<I, F> const Bulk for Mutate<I, F>
where
    I: ~const Bulk,
    F: ~const FnMut(&mut I::Item) + ~const Destruct
{
    fn len(&self) -> usize
    {
        self.bulk.len()
    }
    fn is_empty(&self) -> bool
    {
        self.bulk.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        let Self { bulk, f: mut inspect } = self;
        bulk.first().map(Mutator(&mut inspect))
    }

    fn for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: mutate } = self;

        bulk.for_each(Closure {
            mutate,
            f
        });
    }
    fn try_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, f: mutate } = self;

        bulk.try_for_each(Closure {
            mutate,
            f
        })
    }
}
impl<I, F> const DoubleEndedBulk for Mutate<I, F>
where
    I: ~const DoubleEndedBulk,
    F: ~const FnMut(&mut I::Item) + ~const Destruct
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: mutate } = self;

        bulk.rev_for_each(Closure {
            mutate,
            f
        });
    }
    fn try_rev_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, f: mutate } = self;

        bulk.try_rev_for_each(Closure {
            mutate,
            f
        })
    }
}
unsafe impl<I, F> StaticBulk for Mutate<I, F>
where
    I: StaticBulk,
    F: FnMut(&mut I::Item)
{
    type Array<U> = I::Array<U>;
}
impl<I, F, L> const SplitBulk<L> for Mutate<I, F>
where
    I: ~const SplitBulk<L, Left: ~const Bulk, Right: ~const Bulk>,
    F: FnMut(&mut I::Item) + ~const Clone,
    L: LengthSpec
{
    type Left = Mutate<I::Left, F>;
    type Right = Mutate<I::Right, F>;

    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { bulk, f } = self;
        let (left, right) = bulk.split_at(n);
        (
            left.mutate(f.clone()),
            right.mutate(f)
        )
    }
}

struct Closure<F, FF>
{
    mutate: F,
    f: FF
}
impl<F, FF, T, R> const FnOnce<(T,)> for Closure<F, FF>
where
    F: ~const FnOnce(&mut T),
    FF: ~const FnOnce(T) -> R
{
    type Output = R;

    extern "rust-call" fn call_once(self, (mut x,): (T,)) -> Self::Output
    {
        (self.mutate)(&mut x);
        (self.f)(x)
    }
}
impl<F, FF, T, R> const FnMut<(T,)> for Closure<F, FF>
where
    F: ~const FnMut(&mut T),
    FF: ~const FnMut(T) -> R
{
    extern "rust-call" fn call_mut(&mut self, (mut x,): (T,)) -> Self::Output
    {
        (self.mutate)(&mut x);
        (self.f)(x)
    }
}