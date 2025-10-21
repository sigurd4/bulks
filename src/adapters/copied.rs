use core::marker::Destruct;

use crate::{Bulk, DoubleEndedBulk, StaticBulk};

/// A bulk that copies the elements of an underlying bulk.
///
/// This `struct` is created by the [`copied`](Bulk::copied) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Copied<I>
where
    I: Bulk,
    core::iter::Copied<I::IntoIter>: Iterator<Item: Copy>
{
    bulk: I,
}

impl<'a, I, T> Copied<I>
where
    I: Bulk<Item = &'a T>,
    T: Copy + 'a
{
    pub(crate) const fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }
}

impl<'a, I, T> const Default for Copied<I>
where
    I: ~const Bulk<Item = &'a T> + ~const Default,
    T: Copy + 'a
{
    fn default() -> Self
    {
        I::default().copied()
    }
}

impl<'a, I, T> IntoIterator for Copied<I>
where
    I: Bulk<Item = &'a T>,
    T: Copy + 'a
{
    type IntoIter = core::iter::Copied<I::IntoIter>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk } = self;
        bulk.into_iter().copied()
    }
}
impl<'a, I, T> const Bulk for Copied<I>
where
    I: ~const Bulk<Item = &'a T>,
    T: Copy + 'a
{
    fn len(&self) -> usize
    {
        let Self { bulk } = self;
        bulk.len()
    }

    fn is_empty(&self) -> bool
    {
        let Self { bulk } = self;
        bulk.is_empty()
    }
    
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk } = self;
        bulk.for_each(Closure {
            f
        })
    }
    
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk } = self;
        bulk.try_for_each(Closure {
            f
        })
    }
}
impl<'a, I, T> const DoubleEndedBulk for Copied<I>
where
    I: ~const DoubleEndedBulk<Item = &'a T>,
    T: Copy + 'a
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk } = self;
        bulk.rev_for_each(Closure {
            f
        })
    }
    
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk } = self;
        bulk.try_rev_for_each(Closure {
            f
        })
    }
}
impl<'a, I, T, const N: usize> StaticBulk for Copied<I>
where 
    I: StaticBulk<Item = &'a T, Array<&'a T> = [&'a T; N]>,
    T: Copy + 'a
{
    type Array<U> = [U; N];
}

struct Closure<F>
{
    f: F
}
impl<F, T, R> const FnOnce<(&T,)> for Closure<F>
where
    F: ~const FnOnce(T) -> R,
    T: Copy
{
    type Output = R;

    extern "rust-call" fn call_once(self, (x,): (&T,)) -> Self::Output
    {
        (self.f)(*x)
    }
}
impl<F, T, R> const FnMut<(&T,)> for Closure<F>
where
    F: ~const FnMut(T) -> R,
    T: Copy
{
    extern "rust-call" fn call_mut(&mut self, (x,): (&T,)) -> Self::Output
    {
        (self.f)(*x)
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = ['v', 'e', 'r', 'y', 'c', 'o', 'o', 'l'];
        let a = a.bulk().collect_array();
        let b = a.into_bulk().copied().collect_array();

        println!("{b:?}")
    }
}