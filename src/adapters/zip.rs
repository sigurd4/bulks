use core::fmt;

use array_trait::length::{self, LengthValue};

use crate::{Bulk, ContainedIntoIter, DoubleEndedBulk, IntoBulk, IntoContained, IntoContainedBy, RandomAccessBulk, InplaceBulk, InplaceMutSpec, RandomAccessBulkSpec, SplitBulk};

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
/// let s: [_; _] = bulk.collect();
/// assert_eq!(s, [(1, 4), (2, 5), (3, 6)]);
///
/// // Nested zips are also possible:
/// let zs = [7, 8, 9];
///
/// let bulk = bulks::zip(bulks::zip(xs, ys), zs);
///
/// let s: [_; _] = bulk.collect();
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
    type MinLength = length::Min<A::MinLength, B::MinLength>;
    type MaxLength = length::Min<A::MaxLength, B::MaxLength>;

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
impl<A, B, L> const SplitBulk<L> for Zip<A, B>
where
    A: ~const SplitBulk<L, Left: ~const Bulk, Right: ~const Bulk>,
    B: ~const SplitBulk<L, Left: ~const Bulk, Right: ~const Bulk>,
    Self: ~const Bulk,
    Zip<A::Left, B::Left>: ~const Bulk<Item = Self::Item>,
    Zip<A::Right, B::Right>: ~const Bulk<Item = Self::Item>,
    L: LengthValue
{
    type Left = Zip<A::Left, B::Left>;
    type Right = Zip<A::Right, B::Right>;

    fn split_at(Self { a, b }: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let (a_left, a_right) = a.split_at(n);
        let (b_left, b_right) = b.split_at(n);
        
        (
            a_left.zip(b_left),
            a_right.zip(b_right)
        )
    }
}

impl<'a, A, B> const RandomAccessBulk<'a> for Zip<A, B>
where
    Self: ~const Bulk,
    A: ~const RandomAccessBulk<'a>,
    B: ~const RandomAccessBulk<'a>,
    Zip<A::EachRef, B::EachRef>: ~const Bulk<Item = (A::ItemRef, B::ItemRef)>
{
    type ItemRef = (A::ItemRef, B::ItemRef);
    type EachRef = Zip<A::EachRef, B::EachRef>;

    fn each_ref(Self { a, b }: &'a Self) -> Self::EachRef
    {
        a.each_ref()
            .zip(b.each_ref())
    }
}
impl<'a, A, B> const InplaceBulk<'a> for Zip<A, B>
where
    Self: ~const Bulk,
    A: ~const InplaceBulk<'a>,
    B: ~const InplaceBulk<'a>,
    Zip<A::EachRef, B::EachRef>: ~const Bulk<Item = (A::ItemRef, B::ItemRef)>,
    Zip<A::EachMut, B::EachMut>: ~const Bulk<Item = (A::ItemMut, B::ItemMut)>
{
    type ItemMut = (A::ItemMut, B::ItemMut);
    type EachMut = Zip<A::EachMut, B::EachMut>;

    fn each_mut(Self { a, b }: &'a mut Self) -> Self::EachMut
    {
        a.each_mut()
            .zip(b.each_mut())
    }
}

impl<'a, A, B> const RandomAccessBulkSpec<'a> for Zip<A, B>
where
    Self: ~const Bulk,
    A: ~const RandomAccessBulk<'a>,
    B: ~const RandomAccessBulk<'a>,
    Zip<A::EachRef, B::EachRef>: ~const Bulk<Item = (A::ItemRef, B::ItemRef)>
{
    fn _get<L>(Self { a, b }: &'a Self, i: L) -> Option<Self::ItemRef>
    where
        L: LengthValue
    {
        Some((a.get(i)?, b.get(i)?))
    }
}
impl<'a, A, B> const InplaceMutSpec<'a> for Zip<A, B>
where
    Self: ~const Bulk,
    A: ~const InplaceBulk<'a>,
    B: ~const InplaceBulk<'a>,
    Zip<A::EachRef, B::EachRef>: ~const Bulk<Item = (A::ItemRef, B::ItemRef)>,
    Zip<A::EachMut, B::EachMut>: ~const Bulk<Item = (A::ItemMut, B::ItemMut)>
{
    fn _get_mut<L>(Self { a, b }: &'a mut Self, i: L) -> Option<Self::ItemMut>
    where
        L: LengthValue
    {
        Some((a.get_mut(i)?, b.get_mut(i)?))
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
        let c = bulk.collect::<[_; _], _>();
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
        
        let c = zipped.collect::<[_; _], _>();
        assert_eq!(c, [(4, 6), (6, 8)]);
    }
}