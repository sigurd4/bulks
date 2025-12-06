use core::{borrow::Borrow, marker::Destruct, ops::Try};

use array_trait::length::{self, LengthValue};

use crate::{Bulk, Chain, DoubleEndedBulk, IntoContained, Once, RandomAccessBulk, SplitBulk};

/// A bulk adapter that places a separator between all elements.
///
/// This `struct` is created by [`Bulk::intersperse`]. See its documentation
/// for more information.
#[derive(Debug, Clone)]
pub struct Intersperse<I>
where
    I: Bulk<Item: Clone>
{
    bulk: I,
    separator: I::Item
}

impl<I, T> Intersperse<I>
where
    I: Bulk<Item = T>,
    T: Clone
{
    pub(crate) const fn new(bulk: I, separator: I::Item) -> Self
    {
        Self {
            bulk,
            separator
        }
    }
}

impl<I, T> IntoIterator for Intersperse<I>
where
    I: Bulk<Item = T>,
    T: Clone
{
    type Item = I::Item;
    type IntoIter = <<core::iter::Intersperse<I::IntoIter> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, separator } = self;
        unsafe {
            bulk.into_iter()
                .intersperse(separator)
                .into_contained()
                .into_iter()
        }
    }
}
impl<I, T> const Bulk for Intersperse<I>
where
    I: ~const Bulk<Item = T>,
    T: ~const Clone + ~const Destruct
{
    type MinLength = length::Interspersed<I::MinLength>;
    type MaxLength = length::Interspersed<I::MaxLength>;

    fn len(&self) -> usize
    {
        let Self { bulk, separator: _ } = self;
        let l = bulk.len();
        l + l.saturating_sub(1)
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, separator: _ } = self;
        bulk.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        let Self { bulk, separator } = self;
        let _ = separator;
        bulk.first()
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, separator } = self;
        bulk.for_each(Closure {
            f,
            separator,
            insert: false
        })
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, separator } = self;
        bulk.try_for_each(TryClosure {
            f,
            separator,
            insert: false
        })
    }
}
impl<I, T> const DoubleEndedBulk for Intersperse<I>
where
    I: ~const DoubleEndedBulk<Item = T>,
    T: ~const Clone + ~const Destruct,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, separator } = self;
        bulk.rev_for_each(Closure {
            f,
            separator,
            insert: false
        })
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, separator } = self;
        bulk.try_rev_for_each(TryClosure {
            f,
            separator,
            insert: false
        })
    }
}
impl<I, T, L> const SplitBulk<L> for Intersperse<I>
where
    I: ~const SplitBulk<usize, Item = T, Left: ~const Bulk, Right: ~const Bulk>,
    T: ~const Clone + ~const Destruct,
    Once<T>: ~const SplitBulk<usize, Item = T, Left: ~const Bulk, Right: ~const Bulk>,
    L: LengthValue
{
    type Left = Chain<Intersperse<I::Left>, <Once<T> as SplitBulk<usize>>::Left>;
    type Right = Chain<<Once<T> as SplitBulk<usize>>::Right, Intersperse<I::Right>>;

    fn split_at(Self { bulk, separator }: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let n = length::value::len(n);
        let m = n.div_ceil(2);
        let (left, right) = bulk.split_at(m);
        let g = crate::once(separator.clone());
        let (left_g, right_g) = g.split_at(n % 2);
        (
            left.intersperse(separator.clone()).chain(left_g),
            right_g.chain(right.intersperse(separator))
        )
    }
}
impl<'a, I, T, R> const RandomAccessBulk<'a> for Intersperse<I>
where
    I: ~const RandomAccessBulk<'a, Item = T, ItemRef = &'a R>,
    T: ~const Clone + ~const Destruct + ~const Borrow<R> + 'a,
    R: 'a
{
    type ItemRef = I::ItemRef;
    type EachRef = Intersperse<I::EachRef>;

    fn each_ref(Self { bulk, separator }: &'a Self) -> Self::EachRef
    {
        bulk.each_ref()
            .intersperse(separator.borrow())
    }
}

struct Closure<F, T>
{
    f: F,
    separator: T,
    insert: bool
}
impl<F, T> const FnOnce<(T,)> for Closure<F, T>
where
    F: ~const FnMut(T) + ~const FnOnce(T),
    T: ~const Destruct
{
    type Output = ();

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        let Self { mut f, separator, insert } = self;
        if insert
        {
            f(separator)
        }
        f.call_once((x,))
    }
}
impl<F, T> const FnMut<(T,)> for Closure<F, T>
where
    F: ~const FnMut(T),
    T: ~const Clone
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let Self { f, separator, insert } = self;
        if core::mem::replace(insert, true)
        {
            f(separator.clone())
        }
        f(x)
    }
}

struct TryClosure<F, T>
{
    f: F,
    separator: T,
    insert: bool
}
impl<F, T, R> const FnOnce<(T,)> for TryClosure<F, T>
where
    T: ~const Destruct,
    F: ~const FnMut(T) -> R + ~const Destruct,
    R: ~const Try<Output = (), Residual: ~const Destruct>
{
    type Output = R;

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        let Self { mut f, separator, insert } = self;
        if insert
        {
            f(separator)?
        }
        f(x)
    }
}
impl<F, T, R> const FnMut<(T,)> for TryClosure<F, T>
where
    T: ~const Clone + ~const Destruct,
    F: ~const FnMut(T) -> R,
    R: ~const Try<Output = (), Residual: ~const Destruct>
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let Self { f, separator, insert } = self;
        if core::mem::replace(insert, true)
        {
            f(separator.clone())?
        }
        f(x)
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = ['H', 'e', 'l', 'l', 'o'];
        let b = '_';

        let bulk = a.into_bulk().intersperse(b);

        let c_ref = bulk.each_ref().collect::<String, _>();
        let c = bulk.collect::<String, _>();

        assert_eq!(c, c_ref);
        println!("{:?}", c);
    }
}