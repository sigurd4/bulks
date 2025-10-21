use core::marker::Destruct;

use crate::{Bulk, DoubleEndedBulk, IntoBulk, IntoContained, StaticBulk};


/// A bulk that links two bulks together, in a chain.
///
/// This `struct` is created by [`chain`] or [`Bulk::chain`]. See their
/// documentation for more.
///
/// # Examples
///
/// ```
/// use bulks::{*, array::Bulk};
///
/// let a1 = [1, 2, 3];
/// let a2 = [4, 5, 6];
/// let bulk: Chain<Bulk<'_, _, _>, Bulk<'_, _, _>> = a1.bulk().chain(a2.bulk());
/// 
/// let a = bulk.collect();
/// 
/// assert_eq!(a, [1, 2, 3, 4, 5, 6]);
/// ```
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Chain<A, B>
where
    A: Bulk,
    B: Bulk<Item = A::Item>
{
    a: A,
    b: B
}
impl<A, B> Chain<A, B>
where
    A: Bulk,
    B: Bulk<Item = A::Item>
{
    pub(crate) const fn new(a: A, b: B) -> Self
    {
        Self { a, b }
    }
}

/// Converts the arguments to bulks and links them together, in a chain.
///
/// See the documentation of [`Bulk::chain`] for more.
///
/// # Examples
///
/// ```
/// use bulks::*;
///
/// let a = [1, 2, 3];
/// let b = [4, 5, 6];
///
/// let mut bulk = bulks::chain(a, b);
///
/// let c = bulk.collect();
/// 
/// assert_eq!(c, [1, 2, 3, 4, 5, 6]);
/// ```
pub const fn chain<A, B>(a: A, b: B) -> Chain<A::IntoBulk, B::IntoBulk>
where
    A: ~const IntoBulk,
    B: ~const IntoBulk<Item = A::Item>
{
    Chain::new(a.into_bulk(), b.into_bulk())
}

impl<A, B, T> IntoIterator for Chain<A, B>
where
    A: Bulk<Item = T>,
    B: Bulk<Item = T>
{
    type Item = T;
    type IntoIter = <<core::iter::Chain<A::IntoIter, B::IntoIter> as IntoContained>::IntoContained as IntoIterator>::IntoIter;
    
    fn into_iter(self) -> Self::IntoIter
    {
        let Self { a, b } = self;
        unsafe {
            a.into_iter()
                .chain(b)
                .into_contained()
                .into_iter()
        }
    }
}
impl<A, B, T> const Bulk for Chain<A, B>
where
    A: ~const Bulk<Item = T>,
    B: ~const Bulk<Item = T> + ~const Destruct
{
    fn len(&self) -> usize
    {
        let Self { a, b } = self;
        a.len() + b.len()
    }
    fn is_empty(&self) -> bool
    {
        let Self { a, b } = self;
        a.is_empty() && b.is_empty()
    }
    
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { a, b } = self;

        a.for_each(&mut f);
        b.for_each(f);
    }
    
    fn try_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        T: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { a, b } = self;

        a.try_for_each(&mut f)?;
        b.try_for_each(f)
    }
}
impl<A, B, T> const DoubleEndedBulk for Chain<A, B>
where
    A: ~const DoubleEndedBulk<Item = T> + ~const Destruct,
    B: ~const DoubleEndedBulk<Item = T>,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { a, b } = self;

        b.rev_for_each(&mut f);
        a.rev_for_each(f);
    }
    
    fn try_rev_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        T: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { a, b } = self;

        b.try_rev_for_each(&mut f)?;
        a.try_rev_for_each(f)
    }
}
impl<A, B, T, const N: usize, const M: usize> StaticBulk for Chain<A, B>
where
    A: StaticBulk<Item = T, Array<T> = [T; N]>,
    B: StaticBulk<Item = T, Array<T> = [T; M]>,
    [(); N + M]:
{
    type Array<U> = [U; N + M];
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let c = const {
            let a = [1, 2, 3];
            let b = [4, 5, 6];
            
            a.into_bulk().chain(b).collect_array()
        };

        println!("{c:?}")
    }
}