use core::{marker::Destruct, ops::Try};

use crate::{Bulk, DoubleEndedBulk, IntoContained, StaticBulk};

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
impl<I, T, const N: usize> StaticBulk for Intersperse<I>
where
    I: StaticBulk<Item = T, Array<T> = [T; N]>,
    T: Clone,
    [(); N + N.saturating_sub(1)]:
{
    type Array<U> = [U; N + N.saturating_sub(1)];
}

struct Closure<F, T>
{
    f: F,
    separator: T,
    insert: bool
}
impl<F, T> const FnOnce<(T,)> for Closure<F, T>
where
    F: ~const FnMut(T) -> () + ~const FnOnce(T) -> (),
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
    F: ~const FnMut(T) -> (),
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
        let c = a.into_bulk().intersperse(b).collect::<String>();

        println!("{:?}", c);
    }
}