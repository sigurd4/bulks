use core::{marker::Destruct, ops::Try};

use array_trait::AsArray;
use currying::Curry;

use crate::{Bulk, DoubleEndedBulk, IntoBulk, IntoContained, StaticBulk, util::{Length, LengthMul, LengthSpec}};

/// A bulk that maps each element to an iterator, and yields the elements
/// of the produced bulks.
///
/// This `struct` is created by [`Bulk::flat_map`]. See its documentation
/// for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct FlatMap<I, U, F>
where
    I: Bulk,
    F: FnMut(I::Item) -> U,
    U: IntoBulk<IntoBulk: StaticBulk>
{
    bulk: I,
    map: F
}

impl<I, U, F> FlatMap<I, U, F>
where
    I: Bulk,
    F: FnMut(I::Item) -> U,
    U: IntoBulk<IntoBulk: StaticBulk>
{
    pub(crate) const fn new(bulk: I, map: F) -> Self
    {
        Self {
            bulk,
            map
        }
    }

    const fn chunk() -> usize
    {
        <<<U as IntoBulk>::IntoBulk as StaticBulk>::Array<U::Item> as AsArray>::LENGTH
    }
}

impl<I, U, F> IntoIterator for FlatMap<I, U, F>
where
    I: Bulk,
    F: FnMut(I::Item) -> U,
    U: IntoBulk<IntoBulk: StaticBulk>
{
    type Item = U::Item;
    type IntoIter = <<core::iter::FlatMap<I::IntoIter, U, F> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, map } = self;
        unsafe {
            bulk.into_iter()
                .flat_map(map)
                .into_contained()
                .into_iter()
        }
    }
}
impl<I, U, F> const Bulk for FlatMap<I, U, F>
where
    I: ~const Bulk<Item: ~const Destruct>,
    F: ~const FnMut(I::Item) -> U + ~const Destruct,
    U: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk>
{
    type MinLength<V> = <<<I::MinLength<V> as Length>::LengthSpec as LengthMul<<<U::IntoBulk as StaticBulk>::Array<V> as Length>::LengthSpec>>::LengthMul as LengthSpec>::Length<V>;
    type MaxLength<V> = <<<I::MaxLength<V> as Length>::LengthSpec as LengthMul<<<U::IntoBulk as StaticBulk>::Array<V> as Length>::LengthSpec>>::LengthMul as LengthSpec>::Length<V>;

    fn len(&self) -> usize
    {
        let Self { bulk, map: _ } = self;
        bulk.len()*Self::chunk()
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, map: _ } = self;
        Self::chunk() == 0 || bulk.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        const fn into_first<F, T, U>(f: F, x: T) -> Option<U::Item>
        where
            F: ~const FnOnce(T) -> U,
            U: ~const IntoBulk<Item: ~const Destruct>
        {
            f(x).into_bulk().first()
        }

        let Self { bulk, mut map } = self;
        bulk.first().and_then(into_first.curry(&mut map))
    }
    
    fn for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F, FF>
        {
            map: F,
            f: FF
        }
        impl<F, FF, U, T> const FnOnce<(T,)> for Closure<F, FF>
        where
            F: ~const FnOnce(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk>,
            FF: ~const FnMut(U::Item) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().for_each(self.f)
            }
        }
        impl<F, FF, U, T> const FnMut<(T,)> for Closure<F, FF>
        where
            F: ~const FnMut(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk>,
            FF: ~const FnMut(U::Item) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().for_each(&mut self.f)
            }
        }

        let Self { bulk, map } = self;
        bulk.for_each(Closure {
            map,
            f
        })
    }
    fn try_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F, FF>
        {
            map: F,
            f: FF
        }
        impl<F, FF, U, T, R> const FnOnce<(T,)> for Closure<F, FF>
        where
            F: ~const FnOnce(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk, Item: ~const Destruct>,
            FF: ~const FnMut(U::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().try_for_each(self.f)
            }
        }
        impl<F, FF, U, T, R> const FnMut<(T,)> for Closure<F, FF>
        where
            F: ~const FnMut(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk, Item: ~const Destruct>,
            FF: ~const FnMut(U::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().try_for_each(&mut self.f)
            }
        }

        let Self { bulk, map } = self;
        bulk.try_for_each(Closure {
            map,
            f
        })
    }
}
impl<I, U, F> const DoubleEndedBulk for FlatMap<I, U, F>
where
    I: ~const DoubleEndedBulk<Item: ~const Destruct>,
    F: ~const FnMut(I::Item) -> U + ~const Destruct,
    U: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk>,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F, FF>
        {
            map: F,
            f: FF
        }
        impl<F, FF, U, T> const FnOnce<(T,)> for Closure<F, FF>
        where
            F: ~const FnOnce(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk>,
            FF: ~const FnMut(U::Item) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().rev_for_each(self.f)
            }
        }
        impl<F, FF, U, T> const FnMut<(T,)> for Closure<F, FF>
        where
            F: ~const FnMut(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk>,
            FF: ~const FnMut(U::Item) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().rev_for_each(&mut self.f)
            }
        }

        let Self { bulk, map } = self;
        bulk.rev_for_each(Closure {
            map,
            f
        })
    }
    fn try_rev_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F, FF>
        {
            map: F,
            f: FF
        }
        impl<F, FF, U, T, R> const FnOnce<(T,)> for Closure<F, FF>
        where
            F: ~const FnOnce(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk, Item: ~const Destruct>,
            FF: ~const FnMut(U::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().try_rev_for_each(self.f)
            }
        }
        impl<F, FF, U, T, R> const FnMut<(T,)> for Closure<F, FF>
        where
            F: ~const FnMut(T) -> U,
            U: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk, Item: ~const Destruct>,
            FF: ~const FnMut(U::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                (self.map)(x).into_bulk().try_rev_for_each(&mut self.f)
            }
        }

        let Self { bulk, map } = self;
        bulk.try_rev_for_each(Closure {
            map,
            f
        })
    }
}
unsafe impl<I, U, F, T, V, const N: usize> StaticBulk for FlatMap<I, U, F>
where
    I: StaticBulk<Item = T>,
    F: FnMut(T) -> U,
    U: IntoBulk<Item = V, IntoBulk: StaticBulk<Item = V>>,
    Self: Bulk<MinLength<Self::Item> = [Self::Item; N], MaxLength<Self::Item> = [Self::Item; N]>
{
    type Array<W> = [W; N];
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3];
        let b = a.into_bulk()
            .flat_map(|x| [x, -x])
            .collect::<[_; _]>();

        println!("{b:?}")
    }
}