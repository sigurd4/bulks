use core::{marker::Destruct, ops::{Add, Try}, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};
use currying::Curry;

use crate::{Bulk, DoubleEndedBulk, RandomAccessBulk, InplaceBulk, InplaceBulkSpec, RandomAccessBulkSpec, SplitBulk};

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
    T: ~const SplitBulk<L, Item: ~const Destruct, Left: ~const Bulk, Right: ~const Bulk>,
    N: Length<Elem = (), Value = NN> + ?Sized,
    NN: LengthValue<Metadata = <N as Pointee>::Metadata, Length<()> = N, SaturatingAdd<M> = L, SaturatingSub<L> = R>,
    M: LengthValue,
    L: LengthValue,
    R: LengthValue
{
    type Left = Skip<T::Left, N>;
    type Right = Skip<T::Right, R::Length<()>>;

    fn split_at(Self { bulk, n }: Self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let n = NN::from_metadata(n);
        let l = length::value::saturating_add(n, m);
        let (left, right) = bulk.split_at(l);
        (
            left.skip(n),
            right.skip(length::value::saturating_sub(n, l))
        )
    }
}
impl<T, N> const RandomAccessBulk for Skip<T, N>
where
    T: ~const RandomAccessBulk<Item: ~const Destruct>,
    N: Length<Elem = (), Metadata: ~const Destruct> + ?Sized
{
    type ItemPointee = T::ItemPointee;
    type EachRef<'a> = Skip<T::EachRef<'a>, N>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(Self { bulk, n }: &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        bulk.each_ref().skip(length::value::from_metadata::<N::Value>(*n))
    }
}
impl<T, N> const InplaceBulk for Skip<T, N>
where
    T: ~const InplaceBulk<Item: ~const Destruct>,
    N: Length<Elem = (), Metadata: ~const Destruct> + ?Sized
{
    type EachMut<'a> = Skip<T::EachMut<'a>, N>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_mut<'a>(Self { bulk, n }: &'a mut Self) -> Self::EachMut<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        bulk.each_mut().skip(length::value::from_metadata::<N::Value>(*n))
    }
}
impl<T, N> const RandomAccessBulkSpec for Skip<T, N>
where
    T: ~const RandomAccessBulk,
    N: Length<Elem = ()> + ?Sized
{
    fn _get<'a, L>(Self { bulk, n }: &'a Self, i: L) -> Option<&'a <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: 'a
    {
        bulk.get(length::value::add(length::value::from_metadata::<N::Value>(*n), i))
    }

    fn _get_many<'a, NN, const M: usize>(Self { bulk, n }: &'a Self, i: NN) -> [Option<&'a <Self as RandomAccessBulk>::ItemPointee>; M]
    where
        Self: 'a,
        NN: ~const crate::IntoBulk<Item = usize, IntoBulk: ~const Bulk + crate::StaticBulk<Array<()> = [(); M]>>
    {
        bulk.get_many(i.into_bulk().map(Add::add.curry(length::value::len(length::value::from_metadata::<N::Value>(*n)))))
    }
}
impl<T, N> const InplaceBulkSpec for Skip<T, N>
where
    T: ~const InplaceBulk,
    N: Length<Elem = ()> + ?Sized
{
    fn _get_mut<'a, L>(Self { bulk, n }: &'a mut Self, i: L) -> Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: 'a
    {
        bulk.get_mut(length::value::add(length::value::from_metadata::<N::Value>(*n), i))
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