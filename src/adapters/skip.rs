use core::{marker::Destruct, ops::Try, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};

use crate::{Bulk, DoubleEndedBulk, SplitBulk};

/// A bulk that skips over `n` elements of `bulk`.
///
/// This `struct` is created by the [`skip`](Bulk::skip) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Skip<T, N = [()]>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    bulk: T,
    n: <N as Pointee>::Metadata
}

impl<T, N> Skip<T, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    pub(crate) const fn new(bulk: T, n: N::Value) -> Skip<T, N>
    {
        Self { bulk, n: length::value::into_metadata(n) }
    }
}
impl<T, N> IntoIterator for Skip<T, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    type Item = T::Item;
    type IntoIter = core::iter::Skip<T::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, n } = self;
        bulk.into_iter()
            .skip(length::len_metadata::<N>(n))
    }
}
impl<T, N> const Bulk for Skip<T, N>
where
    T: ~const Bulk<Item: ~const Destruct>,
    N: Length<Elem = ()> + ?Sized
{
    type MinLength = length::SaturatingSub<T::MinLength, N>;
    type MaxLength = length::SaturatingSub<T::MaxLength, N>;

    fn len(&self) -> usize
    {
        let Self { bulk, n } = self;
        bulk.len().saturating_sub(length::len_metadata::<N>(*n))
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, n } = self;
        bulk.len() > length::len_metadata::<N>(*n)
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
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n } = self;
                if n == 0
                {
                    f(x)
                }
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
                    f(x)
                }
                else
                {
                    *n -= 1
                }
            }
        }

        let Self { bulk, n } = self;
        bulk.for_each(Closure {
            f,
            n: length::len_metadata::<N>(n)
        })
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
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n } = self;
                if n == 0
                {
                    f(x)?
                }
                R::from_output(())
            }
        }
        impl<F, T, R> const FnMut<(T,)> for Closure<F>
        where
            T: ~const Destruct,
            F: ~const FnMut(T) -> R,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n } = self;
                if *n == 0
                {
                    f(x)?
                }
                else
                {
                    *n -= 1
                }
                R::from_output(())
            }
        }

        let Self { bulk, n } = self;
        bulk.try_for_each(Closure {
            f,
            n: length::len_metadata::<N>(n)
        })
    }
}
impl<T, N> const DoubleEndedBulk for Skip<T, N>
where
    T: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
    N: Length<Elem = ()> + ?Sized
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, n } = self;
        let m = bulk.len() - length::len_metadata::<N>(n);
        bulk.rev().take(m).for_each(f)
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, n } = self;
        let m = bulk.len() - length::len_metadata::<N>(n);
        bulk.rev().take(m).try_for_each(f)
    }
}
impl<T, N, NN, M, L, R> const SplitBulk<M> for Skip<T, N>
where
    T: ~const SplitBulk<L, Left: ~const Bulk, Right: ~const Bulk>,
    N: Length<Elem = (), Value = NN> + ?Sized,
    NN: LengthValue<Metadata = <N as Pointee>::Metadata, Length<()> = N, SaturatingAdd<M> = L, SaturatingSub<L> = R>,
    M: LengthValue,
    L: LengthValue,
    R: LengthValue
{
    type Left = Skip<T::Left, N>;
    type Right = Skip<T::Right, R::Length<()>>;

    fn split_at(self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { bulk, n } = self;
        let n = NN::from_metadata(n);
        let l = length::value::saturating_add(n, m);
        let (left, right) = bulk.split_at(l);
        (
            left.skip(n),
            right.skip(length::value::saturating_sub(n, l))
        )
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3, 4, 5, 6, 7];
        let (a, b) = a.into_bulk()
            .skip([(); 2])
            .split_at([(); 2]);
        let a = a.collect_array();
        let b = b.collect_array();

        println!("a = {a:?}");
        println!("b = {b:?}");
    }
}