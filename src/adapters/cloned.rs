use core::marker::Destruct;

use crate::{util::LengthSpec, Bulk, DoubleEndedBulk, StaticBulk};

/// A bulk that clones the elements of an underlying bulk.
///
/// This `struct` is created by the [`cloned`](Bulk::cloned) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Cloned<I>
where
    I: Bulk,
    core::iter::Cloned<I::IntoIter>: Iterator<Item: Clone>
{
    bulk: I,
}

impl<'a, I, T> Cloned<I>
where
    I: Bulk<Item = &'a T>,
    T: Clone + 'a
{
    pub(crate) const fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }
}

impl<'a, I, T> const Default for Cloned<I>
where
    I: ~const Bulk<Item = &'a T> + ~const Default,
    T: Clone + 'a
{
    fn default() -> Self
    {
        I::default().cloned()
    }
}

impl<'a, I, T> IntoIterator for Cloned<I>
where
    I: Bulk<Item = &'a T>,
    T: Clone + 'a
{
    type IntoIter = core::iter::Cloned<I::IntoIter>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk } = self;
        bulk.into_iter().cloned()
    }
}
impl<'a, I, T> const Bulk for Cloned<I>
where
    I: ~const Bulk<Item = &'a T>,
    T: ~const Clone + 'a
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

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        let Self { bulk } = self;
        bulk.first().map(Clone::clone)
    }
    fn last(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        let Self { bulk } = self;
        bulk.last().map(Clone::clone)
    }
    fn nth<L>(self, n: L) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        L: ~const LengthSpec
    {
        let Self { bulk } = self;
        bulk.nth(n).map(Clone::clone)
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
impl<'a, I, T> const DoubleEndedBulk for Cloned<I>
where
    I: ~const DoubleEndedBulk<Item = &'a T>,
    T: ~const Clone + 'a
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
unsafe impl<'a, I, T, const N: usize> StaticBulk for Cloned<I>
where 
    I: StaticBulk<Item = &'a T, Array<&'a T> = [&'a T; N]>,
    T: Clone + 'a
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
    T: ~const Clone
{
    type Output = R;

    extern "rust-call" fn call_once(self, (x,): (&T,)) -> Self::Output
    {
        (self.f)(x.clone())
    }
}
impl<F, T, R> const FnMut<(&T,)> for Closure<F>
where
    F: ~const FnMut(T) -> R,
    T: ~const Clone
{
    extern "rust-call" fn call_mut(&mut self, (x,): (&T,)) -> Self::Output
    {
        (self.f)(x.clone())
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = ["it", "works", "right?"];
        let a = a.into_bulk().map(|s| s.to_string()).collect::<[_; _]>();
        let a = a.bulk().collect::<[_; _]>();

        let b = a.into_bulk().cloned().collect::<[_; _]>();

        println!("{b:?}")
    }
}