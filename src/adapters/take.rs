use core::{marker::Destruct, ops::{ControlFlow, Try}, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};

use crate::{Bulk, ContainedIntoIter, DoubleEndedBulk, IntoBulk, IntoContained, SplitBulk};

/// Creates a bulk that only delivers the first `n` iterations of `iterable`.
pub const fn take<I, L>(iterable: I, n: L) -> Take<
    <<I as IntoContained>::IntoContained as IntoBulk>::IntoBulk,
    L::Length<()>
>
where
    I: ~const IntoContained,
    L: LengthValue
{
    unsafe {
        Take::new(iterable.into_contained().into_bulk(), n)
    }
}

/// A bulk that only delivers the first `n` iterations of `bulk`.
///
/// This `struct` is created by the [`take`](Bulk::take) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Take<T, N = [()]>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    bulk: T,
    n: <N as Pointee>::Metadata
}

impl<T, N> Take<T, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    pub(crate) const fn new(bulk: T, n: N::Value) -> Take<T, N>
    {
        Self { bulk, n: length::value::into_metadata(n) }
    }
}
impl<T, N> IntoIterator for Take<T, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    type Item = T::Item;
    type IntoIter = <<core::iter::Take<
        <T::IntoIter as ContainedIntoIter>::ContainedIntoIter
    > as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, n } = self;
        unsafe {
            bulk.into_iter()
                .contained_into_iter()
                .take(length::len_metadata::<N>(n))
                .into_contained()
                .into_iter()
        }
    }
}
impl<T, N> const Bulk for Take<T, N>
where
    T: ~const Bulk<Item: ~const Destruct>,
    N: Length<Elem = ()> + ?Sized
{
    type MinLength = length::Min<T::MinLength, N>;
    type MaxLength = length::Min<T::MaxLength, N>;

    fn len(&self) -> usize
    {
        let Self { bulk, n } = self;
        let n = length::len_metadata::<N>(*n);
        Ord::min(bulk.len(), n)
    }
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F>
        {
            f: F,
            n: usize
        }
        impl<F, T> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const Destruct,
            F: ~const FnOnce(T) + ~const Destruct
        {
            type Output = ControlFlow<()>;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n } = self;
                if n == 0
                {
                    return ControlFlow::Break(())
                }
                f(x);
                ControlFlow::Continue(())
            }
        }
        impl<F, T> const FnMut<(T,)> for Closure<F>
        where
            T: ~const Destruct,
            F: ~const FnMut(T)
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n } = self;
                if *n == 0
                {
                    return ControlFlow::Break(())
                }
                *n -= 1;
                f(x);
                ControlFlow::Continue(())
            }
        }

        let Self { bulk, n } = self;
        bulk.try_for_each(Closure {
            f,
            n: length::len_metadata::<N>(n)
        }).into_value()
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F>
        {
            f: F,
            n: usize
        }
        impl<F, T, R> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const Destruct,
            F: ~const FnOnce(T) -> R + ~const Destruct,
            R: ~const Try<Output = ()>
        {
            type Output = ControlFlow<Result<(), R::Residual>>;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n } = self;
                if n == 0
                {
                    return ControlFlow::Break(Ok(()))
                }
                match f(x).branch()
                {
                    ControlFlow::Break(residual) => ControlFlow::Break(Err(residual)),
                    ControlFlow::Continue(()) => ControlFlow::Continue(())
                }
            }
        }
        impl<F, T, R> const FnMut<(T,)> for Closure<F>
        where
            T: ~const Destruct,
            F: ~const FnMut(T) -> R,
            R: ~const Try<Output = ()>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n } = self;
                if *n == 0
                {
                    return ControlFlow::Break(Ok(()))
                }
                *n -= 1;
                match f(x).branch()
                {
                    ControlFlow::Break(residual) => ControlFlow::Break(Err(residual)),
                    ControlFlow::Continue(()) => ControlFlow::Continue(())
                }
            }
        }

        let Self { bulk, n } = self;
        match bulk.try_for_each(Closure {
            f,
            n: length::len_metadata::<N>(n)
        })
        {
            ControlFlow::Break(Err(residual)) => R::from_residual(residual),
            ControlFlow::Continue(()) | ControlFlow::Break(Ok(())) => R::from_output(())
        }
    }
}
impl<T, N> const DoubleEndedBulk for Take<T, N>
where
    T: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
    N: Length<Elem = ()> + ?Sized,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, n } = self;
        let m = bulk.len() - length::len_metadata::<N>(n);
        bulk.rev().skip(m).for_each(f)
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, n } = self;
        let m = bulk.len() - length::len_metadata::<N>(n);
        bulk.rev().skip(m).try_for_each(f)
    }
}
impl<T, N, NN, M, R> const SplitBulk<M> for Take<T, N>
where
    T: ~const SplitBulk<M, Left: ~const Bulk, Right: ~const Bulk>,
    N: Length<Elem = (), Value = NN> + ?Sized,
    NN: LengthValue<Metadata = N::Metadata, Length<()> = N, SaturatingSub<M> = R>,
    M: LengthValue,
    R: LengthValue
{
    type Left = Take<T::Left, N>;
    type Right = Take<T::Right, R::Length<()>>;

    fn split_at(self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { bulk, n } = self;
        let n = NN::from_metadata(n);
        let (left, right) = bulk.split_at(m);
        (
            left.take(n),
            right.take(length::value::saturating_sub(n, m))
        )
    }
}
// TODO: random-access

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::take(0..6, [(); 10]).collect::<Vec<_>, _>();

        println!("{a:?}")
    }
}