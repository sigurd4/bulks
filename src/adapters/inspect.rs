use core::{fmt, marker::Destruct, ops::Try};

use crate::{Bulk, DoubleEndedBulk, StaticBulk};

/// A bulk that calls a function with a reference to each element before
/// yielding it.
///
/// This `struct` is created by the [`inspect`](Bulk::inspect) method on [`Bulk`]. See its
/// documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Inspect<I, F>
where
    I: Bulk,
    F: FnMut(&I::Item)
{
    bulk: I,
    f: F
}

impl<I, F> Inspect<I, F>
where
    I: Bulk,
    F: FnMut(&I::Item)
{
    pub(crate) const fn new(bulk: I, f: F) -> Self
    {
        Self {
            bulk,
            f
        }
    }
}

impl<I, F> fmt::Debug for Inspect<I, F>
where
    I: Bulk + fmt::Debug,
    F: FnMut(&I::Item)
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let Self { bulk, f: _ } = self;
        f.debug_struct("Inspect").field("bulk", bulk).finish()
    }
}

impl<I, F> IntoIterator for Inspect<I, F>
where
    I: Bulk,
    F: FnMut(&I::Item)
{
    type Item = I::Item;
    type IntoIter = core::iter::Inspect<I::IntoIter, F>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter().inspect(f)
    }
}
impl<I, F> const Bulk for Inspect<I, F>
where
    I: ~const Bulk,
    F: ~const FnMut(&I::Item) + ~const Destruct
{
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
        let Self { bulk, f: mut inspect } = self;
        bulk.first().inspect(&mut inspect)
    }

    fn for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: inspect } = self;
        bulk.for_each(Closure {
            inspect,
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
        let Self { bulk, f: inspect } = self;
        bulk.try_for_each(Closure {
            inspect,
            f
        })
    }
}
impl<I, F> const DoubleEndedBulk for Inspect<I, F>
where
    I: ~const DoubleEndedBulk,
    F: ~const FnMut(&I::Item) + ~const Destruct
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: inspect } = self;
        bulk.rev_for_each(Closure {
            inspect,
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
        let Self { bulk, f: inspect } = self;
        bulk.try_rev_for_each(Closure {
            inspect,
            f
        })
    }
}

unsafe impl<I, F> StaticBulk for Inspect<I, F>
where
    I: StaticBulk,
    F: FnMut(&I::Item)
{
    type Array<U> = I::Array<U>;
}

struct Closure<F, FF>
{
    inspect: F,
    f: FF
}
impl<F, FF, T, R> const FnOnce<(T,)> for Closure<F, FF>
where
    F: ~const FnOnce(&T),
    FF: ~const FnOnce(T) -> R
{
    type Output = R;

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        (self.inspect)(&x);
        (self.f)(x)
    }
}
impl<F, FF, T, R> const FnMut<(T,)> for Closure<F, FF>
where
    F: ~const FnMut(&T),
    FF: ~const FnMut(T) -> R
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        (self.inspect)(&x);
        (self.f)(x)
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = ['0', '1', '2', '3', '4', '5', '6', '7'];

        let b = a.into_bulk()
            .map(|a| a.to_string().parse::<usize>())
            .try_collect_array()
            .unwrap();

        println!("{b:?}");
        
        let c = b.into_bulk()
            .enumerate()
            .inspect(|&(i, a)| assert_eq!(i, a))
            .map(|(_, a)| a.to_string().chars().next())
            .try_collect_array()
            .unwrap();

        assert_eq!(a, c);
    }
}