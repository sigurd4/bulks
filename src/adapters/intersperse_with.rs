use core::{marker::Destruct, ops::Try};

use crate::{Bulk, Chain, DoubleEndedBulk, Flatten, IntoBulk, IntoContained, OnceWith, SplitBulk, StaticBulk, util::LengthSpec};

/// A bulk adapter that places a separator between all elements.
///
/// This `struct` is created by [`Bulk::intersperse_with`]. See its
/// documentation for more information.
#[derive(Debug, Clone)]
pub struct IntersperseWith<I, G>
where
    I: Bulk,
    G: FnMut() -> I::Item
{
    bulk: I,
    separator: G
}

impl<I, G, T> IntersperseWith<I, G>
where
    I: Bulk<Item = T>,
    G: FnMut() -> T
{
    pub(crate) const fn new(bulk: I, separator: G) -> Self
    {
        Self {
            bulk,
            separator
        }
    }
}

impl<I, G, T> IntoIterator for IntersperseWith<I, G>
where
    I: Bulk<Item = T>,
    G: FnMut() -> T
{
    type Item = I::Item;
    type IntoIter = <<core::iter::IntersperseWith<I::IntoIter, G> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, separator } = self;
        unsafe {
            bulk.into_iter()
                .intersperse_with(separator)
                .into_contained()
                .into_iter()
        }
    }
}
impl<I, G, T> const Bulk for IntersperseWith<I, G>
where
    I: ~const Bulk<Item = T>,
    G: ~const FnMut() -> T + ~const Destruct
{
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
        Self::Item: ~const Destruct,
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
        Self::Item: ~const Destruct,
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
impl<I, G, T> const DoubleEndedBulk for IntersperseWith<I, G>
where
    I: ~const DoubleEndedBulk<Item = T>,
    G: ~const FnMut() -> T + Fn() -> T + ~const Destruct,
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
        Self::Item: ~const Destruct,
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
unsafe impl<I, G, T, const N: usize> StaticBulk for IntersperseWith<I, G>
where
    I: StaticBulk<Item = T, Array<T> = [T; N]>,
    G: FnMut() -> T,
    [(); N + N.saturating_sub(1)]:
{
    type Array<U> = [U; N + N.saturating_sub(1)];
}
impl<I, G, T, L> const SplitBulk<L> for IntersperseWith<I, G>
where
    I: ~const SplitBulk<usize, Item = T, Left: ~const Bulk, Right: ~const Bulk>,
    G: ~const FnMut() -> T + ~const FnOnce() -> T + Fn() -> T + ~const Clone + ~const Destruct,
    Option<OnceWith<G>>: ~const IntoBulk<Item = OnceWith<G>>,
    L: ~const LengthSpec
{
    type Left = Chain<IntersperseWith<I::Left, G>, Flatten<<Option<OnceWith<G>> as IntoBulk>::IntoBulk>>;
    type Right = Chain<Flatten<<Option<OnceWith<G>> as IntoBulk>::IntoBulk>, IntersperseWith<I::Right, G>>;

    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { bulk, separator } = self;
        let n = n.len_metadata();
        let m = n.div_ceil(2);
        let (left, right) = bulk.split_at(m);
        let g = crate::once_with(separator.clone());
        let (left_g, right_g) = if n % 2 == 1
        {
            (Some(g), None)
        }
        else
        {
            (None, Some(g))
        };
        (
            left.intersperse_with(separator.clone())
                .chain(left_g.into_bulk()
                    .flatten()
                ),
            right_g.into_bulk()
                .flatten()
                .chain(right.intersperse_with(separator))
        )
    }
}

struct Closure<F, G>
{
    f: F,
    separator: G,
    insert: bool
}
impl<F, G, T> const FnOnce<(T,)> for Closure<F, G>
where
    F: ~const FnMut(T) -> () + ~const FnOnce(T) -> (),
    G: ~const FnOnce() -> T + ~const Destruct
{
    type Output = ();

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        let Self { mut f, separator, insert } = self;
        if insert
        {
            f(separator())
        }
        f.call_once((x,))
    }
}
impl<F, G, T> const FnMut<(T,)> for Closure<F, G>
where
    F: ~const FnMut(T) -> (),
    G: ~const FnMut() -> T
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let Self { f, separator, insert } = self;
        if core::mem::replace(insert, true)
        {
            f(separator())
        }
        f(x)
    }
}

struct TryClosure<F, G>
{
    f: F,
    separator: G,
    insert: bool
}
impl<F, G, T, R> const FnOnce<(T,)> for TryClosure<F, G>
where
    T: ~const Destruct,
    F: ~const FnMut(T) -> R + ~const Destruct,
    G: ~const FnOnce() -> T + ~const Destruct,
    R: ~const Try<Output = (), Residual: ~const Destruct>
{
    type Output = R;

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        let Self { mut f, separator, insert } = self;
        if insert
        {
            f(separator())?
        }
        f(x)
    }
}
impl<F, G, T, R> const FnMut<(T,)> for TryClosure<F, G>
where
    T: ~const Destruct,
    F: ~const FnMut(T) -> R,
    G: ~const FnMut() -> T,
    R: ~const Try<Output = (), Residual: ~const Destruct>
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let Self { f, separator, insert } = self;
        if core::mem::replace(insert, true)
        {
            f(separator())?
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
        let c = a.into_bulk().intersperse_with(|| b).collect::<String>();

        println!("{:?}", c);
    }
}