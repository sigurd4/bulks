use core::{marker::Destruct, ops::{Mul, Try}, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};
use currying::Curry;

use crate::{Bulk, RandomAccessBulk, InplaceBulk, InplaceBulkSpec, RandomAccessBulkSpec, SplitBulk};

/// A bulk that steps by a custom amount.
///
/// This `struct` is created by the [`step_by`](Bulk::step_by) method on [`Bulk`]. See
/// its documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct StepBy<T, N = [()]>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    bulk: T,
    step: <N as Pointee>::Metadata
}

impl<T, N> StepBy<T, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    pub(crate) const fn new(bulk: T, step: N::Value) -> StepBy<T, N>
    {
        let step = length::value::into_metadata(step);
        assert!(length::len_metadata::<N>(step) != 0);
        Self { bulk, step }
    }
}

impl<T, N> IntoIterator for StepBy<T, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized
{
    type Item = T::Item;
    type IntoIter = core::iter::StepBy<T::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, step } = self;
        bulk.into_iter()
            .step_by(length::len_metadata::<N>(step))
    }
}
impl<T, N> const Bulk for StepBy<T, N>
where
    T: ~const Bulk<Item: ~const Destruct>,
    N: Length<Elem = ()> + ?Sized
{
    type MinLength = length::DivCeil<T::MinLength, N>;
    type MaxLength = length::DivCeil<T::MaxLength, N>;

    fn len(&self) -> usize
    {
        let Self { bulk, step } = self;
        bulk.len()/length::len_metadata::<N>(*step)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F>
        {
            f: F,
            n: usize,
            step: usize
        }
        impl<F, T> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const Destruct,
            F: ~const FnOnce(T) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                let Self { f, n, step: _ } = self;
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
                let Self { f, n, step } = self;
                if *n == 0
                {
                    f(x)
                }
                *n += 1;
                *n %= *step
            }
        }

        let Self { bulk, step } = self;
        bulk.for_each(Closure {
            f,
            n: 0,
            step: length::len_metadata::<N>(step)
        })
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F>
        {
            f: F,
            n: usize,
            step: usize
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
                let Self { f, n, step: _ } = self;
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
                let Self { f, n, step } = self;
                if *n == 0
                {
                    f(x)?
                }
                *n += 1;
                *n %= *step;
                R::from_output(())
            }
        }

        let Self { bulk, step } = self;
        bulk.try_for_each(Closure {
            f,
            n: 0,
            step: length::len_metadata::<N>(step)
        })
    }
}
impl<T, N, NN, M, L> const SplitBulk<M> for StepBy<T, N>
where
    T: ~const SplitBulk<L, Item: ~const Destruct, Left: ~const Bulk, Right: ~const Bulk>,
    N: Length<Elem = (), Value = NN> + ?Sized,
    NN: LengthValue<Metadata = <N as Pointee>::Metadata, Length<()> = N, SaturatingMul<M> = L>,
    M: LengthValue,
    L: LengthValue
{
    type Left = StepBy<T::Left, N>;
    type Right = StepBy<T::Right, N>;

    fn split_at(Self { bulk, step }: Self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let n = NN::from_metadata(step);
        let (left, right) = bulk.split_at(length::value::saturating_mul(n, m));
        (
            left.step_by(n),
            right.step_by(n)
        )
    }
}

impl<T, N> const RandomAccessBulk for StepBy<T, N>
where
    T: ~const RandomAccessBulk<Item: ~const Destruct>,
    N: Length<Elem = (), Metadata: ~const Destruct> + ?Sized
{
    type ItemPointee = T::ItemPointee;
    type EachRef<'a> = StepBy<T::EachRef<'a>, N>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(Self { bulk, step }: &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        bulk.each_ref().step_by(length::value::from_metadata::<N::Value>(*step))
    }
}
impl<T, N> const InplaceBulk for StepBy<T, N>
where
    T: ~const InplaceBulk<Item: ~const Destruct>,
    N: Length<Elem = (), Metadata: ~const Destruct> + ?Sized
{
    type EachMut<'a> = StepBy<T::EachMut<'a>, N>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_mut<'a>(Self { bulk, step }: &'a mut Self) -> Self::EachMut<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        bulk.each_mut().step_by(length::value::from_metadata::<N::Value>(*step))
    }
}

impl<T, N> const RandomAccessBulkSpec for StepBy<T, N>
where
    T: ~const RandomAccessBulk,
    N: Length<Elem = ()> + ?Sized
{
    fn _get<'a, L>(Self { bulk, step }: &'a Self, i: L) -> Option<&'a <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: 'a
    {
        bulk.get(length::value::mul(length::value::from_metadata::<N::Value>(*step), i))
    }
    fn _get_many<'a, NN, const M: usize>(Self { bulk, step }: &'a Self, i: NN) -> [Option<&'a <Self as RandomAccessBulk>::ItemPointee>; M]
    where
        Self: 'a,
        NN: ~const crate::IntoBulk<Item = usize, IntoBulk: ~const Bulk + crate::StaticBulk<Array<()> = [(); M]>>
    {
        bulk.get_many(i.into_bulk().map(Mul::mul.curry(length::value::len(length::value::from_metadata::<N::Value>(*step)))))
    }
}
impl<T, N> const InplaceBulkSpec for StepBy<T, N>
where
    T: ~const InplaceBulk,
    N: Length<Elem = ()> + ?Sized
{
    fn _get_mut<'a, L>(Self { bulk, step }: &'a mut Self, i: L) -> Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: 'a
    {
        bulk.get_mut(length::value::mul(length::value::from_metadata::<N::Value>(*step), i))
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let a_even = a.into_bulk().step_by([(); 2]).collect::<[_; _], _>();
        println!("{a_even:?}");

        let a_odd = a.into_bulk().skip([(); 1]).step_by([(); 2]).collect::<[_; _], _>();
        println!("{a_odd:?}");
    }
}