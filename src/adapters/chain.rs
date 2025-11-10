use core::marker::Destruct;

use array_trait::length::{self, Length, LengthValue};

use crate::{Bulk, DoubleEndedBulk, IntoBulk, IntoContained, SplitBulk};


/// A bulk that links two bulks together, in a chain.
///
/// This `struct` is created by [`chain`] or [`Bulk::chain`]. See their
/// documentation for more.
///
/// # Examples
///
/// ```
/// # #![feature(generic_const_exprs)]
/// use bulks::*;
///
/// let a1 = [1, 2, 3];
/// let a2 = [4, 5, 6];
/// let bulk = a1.into_bulk().chain(a2.into_bulk());
/// 
/// let a = bulk.collect::<[_; _]>();
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
/// # #![feature(generic_const_exprs)]
/// use bulks::*;
///
/// let a = [1, 2, 3];
/// let b = [4, 5, 6];
///
/// let mut bulk = bulks::chain(a, b);
///
/// let c = bulk.collect::<[_; _]>();
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
    A: ~const Bulk<Item = T> + ~const Destruct,
    B: ~const Bulk<Item = T> + ~const Destruct
{
    type MinLength = length::Add<A::MinLength, B::MinLength>;
    type MaxLength = length::Add<A::MinLength, B::MinLength>;

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
    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        let Self { a, b } = self;
        match a.first()
        {
            Some(first) => Some(first),
            None => b.first()
        }
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
impl<A, B, T, D, L, R> const SplitBulk<L> for Chain<A, B>
where
    A: ~const SplitBulk<L, Item = T, Left: ~const Bulk, Right: ~const Bulk, Length: Length<Value = D>> + ~const Bulk + ~const Destruct,
    B: ~const SplitBulk<R, Item = T, Left: ~const Bulk, Right: ~const Bulk>,
    L: LengthValue<SaturatingSub<D> = R>,
    R: LengthValue,
    D: LengthValue
{
    type Left = Chain<A::Left, B::Left>;
    type Right = Chain<A::Right, B::Right>;

    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { a, b } = self;
        let m = length::value::saturating_sub(n, length::value::or_len::<D>(a.len()));
        let (a_left, a_right) = a.split_at(n);
        let (b_left, b_right) = b.split_at(m);
        (
            a_left.chain(b_left),
            a_right.chain(b_right)
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
        let (a, b) = const {
            let a = [1, 2, 3];
            let b = [4, 5, 6];
            
            let (a, b) = a.into_bulk().chain(b).split_at([(); 4]);
            (a.collect_array(), b.collect_array())
        };

        println!("{a:?} {b:?}")
    }
}