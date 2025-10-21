use core::{marker::Destruct, ops::Try};

use array_trait::AsArray;

use crate::{Bulk, DoubleEndedBulk, IntoBulk, IntoContained, StaticBulk};

/// A bulk that flattens one level of nesting in a of things
/// that can be turned into bulks.
///
/// This `struct` is created by the [`flatten`](Bulk::flatten) method on [`Bulk`]. See its
/// documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    bulk: I
}

impl<I> Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    pub(crate) const fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }
}

impl<I> IntoIterator for Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    type Item = <I::Item as IntoIterator>::Item;
    type IntoIter = <<core::iter::Flatten<I::IntoIter> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk } = self;
        unsafe {
            bulk.into_iter()
                .flatten()
                .into_contained()
                .into_iter()
        }
    }
}
impl<I> const Bulk for Flatten<I>
where
    I: ~const Bulk<Item: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk> + ~const Destruct>
{
    fn len(&self) -> usize
    {
        let Self { bulk } = self;
        bulk.len()*<<<I::Item as IntoBulk>::IntoBulk as StaticBulk>::Array<<I::Item as IntoIterator>::Item> as AsArray>::LENGTH
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk } = self;
        <<<I::Item as IntoBulk>::IntoBulk as StaticBulk>::Array<<I::Item as IntoIterator>::Item> as AsArray>::LENGTH == 0 || bulk.is_empty()
    }
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().for_each(self.f)
            }
        }
        impl<F, T> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.for_each(Closure {
            f
        })
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T, R> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_for_each(self.f)
            }
        }
        impl<F, T, R> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.try_for_each(Closure {
            f
        })
    }
}
impl<I> const DoubleEndedBulk for Flatten<I>
where
    I: ~const DoubleEndedBulk<Item: ~const IntoBulk<IntoBulk: ~const DoubleEndedBulk + StaticBulk> + ~const Destruct>,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().rev_for_each(self.f)
            }
        }
        impl<F, T> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().rev_for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.rev_for_each(Closure {
            f
        })
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T, R> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_rev_for_each(self.f)
            }
        }
        impl<F, T, R> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_rev_for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.try_rev_for_each(Closure {
            f
        })
    }
}
impl<I, T, V, const N: usize, const M: usize> StaticBulk for Flatten<I>
where
    I: StaticBulk<Item = T, Array<T> = [T; N]>,
    T: IntoBulk<Item = V, IntoBulk: StaticBulk<Item = V, Array<V> = [V; M]>>,
    [(); N*M]:
{
    type Array<W> = [W; N*M];
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [[1, -1], [2, -2], [3, -3]];
        let b = a.into_bulk()
            .flatten()
            .collect_array();

        println!("{b:?}")
    }
}