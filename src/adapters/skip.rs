use core::{marker::Destruct, ops::Try, ptr::Pointee};

use crate::{Bulk, DoubleEndedBulk, SplitBulk, StaticBulk, util::{Length, LengthSatAdd, LengthSpec, LengthSatSub}};

/// A bulk that skips over `n` elements of `bulk`.
///
/// This `struct` is created by the [`skip`](Bulk::skip) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Skip<T, N = [<T as IntoIterator>::Item]>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    bulk: T,
    n: <N as Pointee>::Metadata
}

impl<T, N> Skip<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    pub(crate) const fn new(bulk: T, n: N::LengthSpec) -> Skip<T, N>
    {
        Self { bulk, n: n.into_metadata() }
    }
}
impl<T, N> IntoIterator for Skip<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    type Item = T::Item;
    type IntoIter = core::iter::Skip<T::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, n } = self;
        bulk.into_iter()
            .skip(N::len_metadata(n))
    }
}
impl<T, N> const Bulk for Skip<T, N>
where
    T: ~const Bulk<Item: ~const Destruct>,
    N: ~const Length<Elem = T::Item> + ?Sized
{
    type MinLength<U> = <<<T::MinLength<U> as Length>::LengthSpec as LengthSatSub<N::LengthSpec>>::LengthSatSub as LengthSpec>::Length<U>;
    type MaxLength<U> = <<<T::MaxLength<U> as Length>::LengthSpec as LengthSatSub<N::LengthSpec>>::LengthSatSub as LengthSpec>::Length<U>;

    fn len(&self) -> usize
    {
        let Self { bulk, n } = self;
        bulk.len().saturating_sub(N::len_metadata(*n))
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, n } = self;
        bulk.len() > N::len_metadata(*n)
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
            n: N::len_metadata(n)
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
            n: N::len_metadata(n)
        })
    }
}
impl<T, N> const DoubleEndedBulk for Skip<T, N>
where
    T: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
    N: ~const Length<Elem = T::Item> + ?Sized
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, n } = self;
        let m = bulk.len() - N::len_metadata(n);
        bulk.rev().take(m).for_each(f)
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, n } = self;
        let m = bulk.len() - N::len_metadata(n);
        bulk.rev().take(m).try_for_each(f)
    }
}
unsafe impl<T, A, const N: usize, const M: usize> StaticBulk for Skip<T, [A; N]>
where
    T: StaticBulk<Item = A, Array<A> = [A; M]>,
    [A; M.saturating_sub(N)]:
{
    type Array<U> = [U; M.saturating_sub(N)];
}
impl<T, N, NN, M, L, R> const SplitBulk<M> for Skip<T, N>
where
    T: ~const SplitBulk<L, Left: ~const Bulk, Right: ~const Bulk>,
    N: Length<Elem = T::Item, LengthSpec = NN> + ?Sized,
    NN: ~const LengthSpec<Metadata = <N as Pointee>::Metadata, Length<T::Item> = N> + ~const LengthSatAdd<M, LengthSatAdd = L> + ~const LengthSatSub<L, LengthSatSub = R>,
    M: LengthSpec,
    L: LengthSpec,
    R: ~const LengthSpec
{
    type Left = Skip<T::Left, N>;
    type Right = Skip<T::Right, R::Length<T::Item>>;

    fn split_at(self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { bulk, n } = self;
        let n = NN::from_metadata(n);
        let l = n.len_sat_add(m);
        let (left, right) = bulk.split_at(l);
        (
            left.skip(n),
            right.skip(n.len_sat_sub(l))
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