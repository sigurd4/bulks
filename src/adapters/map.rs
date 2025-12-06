use core::{fmt, marker::Destruct};

use array_trait::length::LengthValue;

use crate::{Bulk, DoubleEndedBulk, SplitBulk};

/// A bulk that maps the values of `bulk` with `f`.
///
/// This `struct` is created by the [`map`](Bulk::map) method on [`Bulk`]. See its
/// documentation for more.
///
/// # Notes about side effects
///
/// The [`map`](Bulk::map) iterator implements [`DoubleEndedIterator`](core::iter::DoubleEndedIterator), meaning that
/// you can also [`map`](Bulk::map) backwards:
///
/// ```rust
/// use bulks::*;
/// 
/// let v: [i32; 3] = [1, 2, 3].into_bulk().map(|x| x + 1).rev().collect();
///
/// assert_eq!(v, [4, 3, 2]);
/// ```
///
/// But if your closure has state, iterating backwards may act in a way you do
/// not expect. Let's go through an example. First, in the forward direction:
///
/// ```rust
/// use bulks::*;
/// 
/// let mut c = 0;
///
/// for pair in ['a', 'b', 'c'].into_bulk()
///     .map(|letter| { c += 1; (letter, c) })
/// {
///     println!("{pair:?}");
/// }
/// ```
///
/// This will print `('a', 1), ('b', 2), ('c', 3)`.
///
/// Now consider this twist where we add a call to [`rev`](Bulk::rev). This version will
/// print `('c', 1), ('b', 2), ('a', 3)`. Note that the letters are reversed,
/// but the values of the counter still go in order. This is because `map()` is
/// still being called lazily on each item, but we are popping items off the
/// back of the array now, instead of shifting them from the front.
///
/// ```rust
/// use bulks::*;
/// 
/// let mut c = 0;
///
/// for pair in ['a', 'b', 'c'].into_bulk()
///     .map(|letter| { c += 1; (letter, c) })
///     .rev()
/// {
///     println!("{pair:?}");
/// }
/// ```
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    bulk: I,
    f: F
}

impl<I, F> Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    pub(crate) const fn new(bulk: I, f: F) -> Self
    {
        Self {
            bulk,
            f
        }
    }
}

impl<I, F> fmt::Debug for Map<I, F>
where
    I: Bulk + fmt::Debug,
    F: FnMut<(I::Item,)>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let Self { bulk, f: _ } = self;
        f.debug_struct("Map").field("bulk", bulk).finish()
    }
}

impl<I, F> IntoIterator for Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    type Item = F::Output;
    type IntoIter = core::iter::Map<I::IntoIter, F>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter().map(f)
    }
}
impl<I, F> const Bulk for Map<I, F>
where
    I: ~const Bulk<Item: ~const Destruct>,
    F: ~const FnMut<(I::Item,)> + ~const Destruct
{
    type MinLength = I::MinLength;
    type MaxLength = I::MaxLength;
    
    fn len(&self) -> usize
    {
        let Self { bulk, f: _ } = self;
        bulk.len()
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, f: _ } = self;
        bulk.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        let Self { bulk, mut f } = self;
        bulk.first().map(&mut f)
    }

    fn for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: map } = self;
        bulk.for_each(Closure {
            map,
            f
        })
    }
    fn try_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, f: map } = self;
        bulk.try_for_each(Closure {
            map,
            f
        })
    }
}
impl<I, F> const DoubleEndedBulk for Map<I, F>
where
    I: ~const DoubleEndedBulk<Item: ~const Destruct>,
    F: ~const FnMut<(I::Item,)> + ~const Destruct
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: map } = self;
        bulk.rev_for_each(Closure {
            map,
            f
        })
    }
    fn try_rev_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, f: map } = self;
        bulk.try_rev_for_each(Closure {
            map,
            f
        })
    }
}
impl<I, F, L> const SplitBulk<L> for Map<I, F>
where
    I: ~const SplitBulk<L, Item: ~const Destruct, Left: ~const Bulk, Right: ~const Bulk>,
    F: ~const FnMut<(I::Item,)> + ~const Clone + ~const Destruct,
    L: LengthValue
{
    type Left = Map<I::Left, F>;
    type Right = Map<I::Right, F>;

    fn split_at(Self { bulk, f }: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let (left, right) = bulk.split_at(n);
        (
            left.map(f.clone()),
            right.map(f)
        )
    }
}

struct Closure<M, F>
{
    map: M,
    f: F
}
impl<M, F, T, U, R> const FnOnce<(T,)> for Closure<M, F>
where
    M: ~const FnOnce(T) -> U,
    F: ~const FnOnce(U) -> R
{
    type Output = R;

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        let Self { map, f } = self;
        f(map(x))
    }
}
impl<M, F, T, U, R> const FnMut<(T,)> for Closure<M, F>
where
    M: ~const FnMut(T) -> U,
    F: ~const FnMut(U) -> R
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let Self { map, f } = self;
        f(map(x))
    }
}