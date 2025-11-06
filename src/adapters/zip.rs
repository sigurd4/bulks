use core::fmt;

use crate::{Bulk, ContainedIntoIter, DoubleEndedBulk, IntoBulk, IntoContained, IntoContainedBy, SplitBulk, StaticBulk, util::{Length, LengthMin, LengthSpec}};

/// Converts the arguments to bulks and zips them.
///
/// See the documentation of [`Bulk::zip`](crate::Bulk::zip) for more.
///
/// # Examples
///
/// ```
/// # #![feature(generic_const_exprs)]
/// use bulks::*;
///
/// let xs = [1, 2, 3];
/// let ys = [4, 5, 6];
///
/// let bulk = bulks::zip(xs, ys);
///
/// let s = bulk.collect::<[_; _]>();
/// assert_eq!(s, [(1, 4), (2, 5), (3, 6)]);
///
/// // Nested zips are also possible:
/// let zs = [7, 8, 9];
///
/// let bulk = bulks::zip(bulks::zip(xs, ys), zs);
///
/// let s = bulk.collect::<[_; _]>();
/// assert_eq!(s, [((1, 4), 7), ((2, 5), 8), ((3, 6), 9)]);
/// ```
pub const fn zip<A, B>(a: A, b: B) -> Zip<
    A::IntoBulk,
    <B::IntoContained as IntoBulk>::IntoBulk
>
where
    A: ~const IntoBulk,
    B: ~const IntoContainedBy<A>
{
    unsafe {
        Zip::new(
            a.into_contained().into_bulk(),
            b.into_contained().into_bulk()
        )
    }
}

/// Converts the arguments to bulks and zips them.
///
/// See the documentation of [`Bulk::zip`](crate::Bulk::zip) for more.
pub const fn rzip<A, B>(a: A, b: B) -> Zip<
    <A::IntoContained as IntoBulk>::IntoBulk,
    B::IntoBulk,
>
where
    A: ~const IntoContainedBy<B>,
    B: ~const IntoBulk
{
    unsafe {
        Zip::new(
            a.into_contained().into_bulk(),
            b.into_contained().into_bulk()
        )
    }
}

/// A bulk that operates on two other bulks simultaneously.
///
/// This `struct` is created by [`zip`] or [`Bulk::zip`].
/// See their documentation for more.
#[derive(Clone)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Zip<A, B>
where
    A: Bulk,
    B: Bulk
{
    a: A,
    b: B,
}

impl<A, B> Zip<A, B>
where
    A: Bulk,
    B: Bulk
{
    pub(crate) const fn new(a: A, b: B) -> Zip<A, B>
    {
        Self { a, b }
    }
}

impl<A, B> IntoIterator for Zip<A, B>
where
    A: Bulk,
    B: Bulk
{
    type Item = (A::Item, B::Item);
    type IntoIter = <<core::iter::Zip<
        <A::IntoIter as ContainedIntoIter>::ContainedIntoIter,
        <B::IntoIter as ContainedIntoIter>::ContainedIntoIter
    > as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { a, b } = self;
        unsafe {
            core::iter::zip(
                a.into_iter()
                    .contained_into_iter(),
                b.into_iter()
                    .contained_into_iter()
            ).into_contained()
            .into_iter()
        }
    }
}
impl<A, B> Bulk for Zip<A, B>
where
    A: Bulk,
    B: Bulk
{
    type MinLength<U> = <<<A::MinLength<U> as Length>::LengthSpec as LengthMin<<B::MinLength<U> as Length>::LengthSpec>>::LengthMin as LengthSpec>::Length<U>;
    type MaxLength<U> = <<<A::MaxLength<U> as Length>::LengthSpec as LengthMin<<B::MaxLength<U> as Length>::LengthSpec>>::LengthMin as LengthSpec>::Length<U>;

    fn len(&self) -> usize
    {
        let Self { a, b } = self;
        Ord::min(a.len(), b.len())
    }
    fn is_empty(&self) -> bool
    {
        let Self { a, b } = self;
        a.is_empty() || b.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        let Self { a, b } = self;
        match (a.first(), b.first())
        {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None
        }
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.into_iter().for_each(f)
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        self.into_iter().try_for_each(f)
    }
}
impl<A, B> DoubleEndedBulk for Zip<A, B>
where
    A: DoubleEndedBulk,
    B: DoubleEndedBulk,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.into_iter().rev().for_each(f)
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        self.into_iter().rev().try_for_each(f)
    }
}
unsafe impl<A, B, const N: usize> StaticBulk for Zip<A, B>
where
    A: StaticBulk,
    B: StaticBulk,
    Self: Bulk<MinLength<Self::Item> = [Self::Item; N], MaxLength<Self::Item> = [Self::Item; N]>
{
    type Array<U> = [U; N];
}
impl<A, B, L> const SplitBulk<L> for Zip<A, B>
where
    A: ~const SplitBulk<L, Left: ~const Bulk, Right: ~const Bulk>,
    B: ~const SplitBulk<L, Left: ~const Bulk, Right: ~const Bulk>,
    L: LengthSpec
{
    type Left = Zip<A::Left, B::Left>;
    type Right = Zip<A::Right, B::Right>;

    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { a, b } = self;
        let (a_left, a_right) = a.split_at(n);
        let (b_left, b_right) = b.split_at(n);
        
        (
            a_left.zip(b_left),
            a_right.zip(b_right)
        )
    }
}

impl<A, B> fmt::Debug for Zip<A, B>
where
    A: Bulk + fmt::Debug,
    B: Bulk + fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Zip").field("a", &self.a).field("b", &self.b).finish()
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 3, 5];
        let b = [2, 4, 6];
        let bulk = a.into_bulk()
            .zip(b)
            .map(|(a, b)| a + b);
        let c = bulk.collect::<[_; _], [_; _]>();
        println!("{c:?}")
    }

    #[test]
    fn ugly()
    {
        let a = [1, 2, 3];
        let b = [2, 3, 4];
        
        let zipped = a.into_bulk()
            .map(|x| x * 2)
            .skip([(); 1])
            .zip(b.into_bulk()
                .map(|x| x * 2)
                .skip([(); 1])
            );
        
        let c = zipped.collect::<[_; _]>();
        assert_eq!(c, [(4, 6), (6, 8)]);
    }
}