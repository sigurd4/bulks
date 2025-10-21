use core::{marker::Destruct, ops::Try, ptr::Pointee};

use crate::{util::{Length, LengthSpec}, Bulk, StaticBulk};

/// A bulk that steps by a custom amount.
///
/// This `struct` is created by the [`step_by`](Bulk::step_by) method on [`Bulk`]. See
/// its documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct StepBy<T, N = [<T as IntoIterator>::Item]>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    bulk: T,
    step: <N as Pointee>::Metadata
}

impl<T, N> StepBy<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    pub(crate) const fn new(bulk: T, step: N::LengthSpec) -> StepBy<T, N>
    where
        N: ~const Length<Elem = T::Item>
    {
        let step = step.len_metadata();
        assert!(N::len_metadata(step) != 0);
        Self { bulk, step }
    }
}

impl<T, N> IntoIterator for StepBy<T, N>
where
    T: Bulk,
    N: Length<Elem = T::Item> + ?Sized
{
    type Item = T::Item;
    type IntoIter = core::iter::StepBy<T::IntoIter>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, step } = self;
        bulk.into_iter()
            .step_by(N::len_metadata(step))
    }
}
impl<T, N> const Bulk for StepBy<T, N>
where
    T: ~const Bulk<Item: ~const Destruct>,
    N: ~const Length<Elem = T::Item> + ?Sized
{
    fn len(&self) -> usize
    {
        let Self { bulk, step } = self;
        bulk.len()/N::len_metadata(*step)
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
            step: N::len_metadata(step)
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
            step: N::len_metadata(step)
        })
    }
}
impl<T, A, const N: usize, const M: usize> StaticBulk for StepBy<T, [A; N]>
where
    T: StaticBulk<Item = A, Array<A> = [A; M]>,
    [A; M.div_ceil(N)]:
{
    type Array<U> = [U; M.div_ceil(N)];
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let a_even = a.into_bulk().step_by([(); 2]).collect::<[_; _]>();
        println!("{a_even:?}");

        let a_odd = a.into_bulk().skip([(); 1]).step_by([(); 2]).collect::<[_; _]>();
        println!("{a_odd:?}");
    }
}