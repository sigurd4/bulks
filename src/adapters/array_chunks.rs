use core::{marker::Destruct, ops::{Residual, Try}};

use array_trait::length::{self, LengthValue};

use crate::{Bulk, CollectionAdapter, CollectionStrategy, DoubleEndedBulk, FromBulk, IntoBulk, RandomAccessBulk, InplaceBulk, Rev, SplitBulk, adapters::array_chunks_with_remainder::ArrayChunksWithRemainder, util::{self, ArrayBuffer}};

/// A bulk over `N` elements of the bulk at a time.
///
/// The chunks do not overlap. If `N` does not divide the length of the
/// iterator, then the last up to `N-1` elements will be omitted.
///
/// This `struct` is created by the [`array_chunks`][Bulk::array_chunks]
/// method on [`Bulk`]. See its documentation for more.
#[derive(Debug, Clone)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct ArrayChunks<I, const N: usize>
where
    I: Bulk
{
    bulk: I
}

impl<I, const N: usize> ArrayChunks<I, N>
where
    I: Bulk
{
    #[track_caller]
    pub(crate) const fn new(bulk: I) -> Self
    {
        assert!(N != 0, "chunk size must be non-zero");
        Self {
            bulk
        }
    }

    pub(crate) const fn into_inner(self) -> I
    {
        let Self { bulk } = self;
        bulk
    }

    pub(crate) const fn skip_len<const REV: bool>(&self) -> usize
    where
        I: ~const Bulk
    {
        if REV
        {
            self.bulk.len() % N
        }
        else
        {
            0
        }
    }

    pub const fn for_each_with_remainder<F>(self, f: F) -> <ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk
    where
        I: ~const Bulk<Item: ~const Destruct>,
        F: ~const FnMut(<Self as IntoIterator>::Item) + ~const Destruct,
        ArrayBuffer<I::Item, N, false>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner(), &mut remainder);
        bulk.for_each(f);
        remainder.into_bulk()
    }

    pub const fn try_for_each_with_remainder<F, R, RR>(self, f: F) -> RR
    where
        I: ~const Bulk<Item: ~const Destruct>,
        F: ~const FnMut(<Self as IntoIterator>::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct + Residual<<ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk, TryType = RR>>,
        RR: ~const Try<Output = <ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk, Residual = R::Residual>,
        ArrayBuffer<I::Item, N, false>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner(), &mut remainder);
        bulk.try_for_each(f)?;
        RR::from_output(remainder.into_bulk())
    }

    #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
    pub const fn collect_with_remainder<C, A>(self) -> (C, <util::ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk)
    where
        Self: Sized,
        I: ~const Bulk<Item: ~const Destruct>,
        C: ~const FromBulk<A>,
        A: CollectionAdapter<Elem = [I::Item; N]> + for<'a> ~const CollectionStrategy<ArrayChunksWithRemainder::<'a, I, N, false>, C> + ?Sized,
        util::ArrayBuffer<I::Item, N, false>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner(), &mut remainder);
        let collection = bulk.collect();
        (
            collection,
            remainder.into_bulk()
        )
    }
}

impl<I, const N: usize> Rev<ArrayChunks<I, N>>
where
    I: DoubleEndedBulk
{
    pub const fn for_each_with_remainder<F>(self, f: F) -> <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk
    where
        ArrayChunks<I, N>: Sized,
        I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
        F: ~const FnMut(<Self as IntoIterator>::Item) + ~const Destruct,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner().into_inner().rev(), &mut remainder);
        bulk.for_each(f);
        remainder.into_bulk()
    }

    pub const fn try_for_each_with_remainder<F, R>(self, f: F) -> <<R as Try>::Residual as Residual<<ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk>>::TryType
    where
        ArrayChunks<I, N>: Sized,
        I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
        F: ~const FnMut(<ArrayChunks<I, N> as IntoIterator>::Item) -> R + ~const Destruct,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk,
        R: ~const Try<Output = (), Residual: Residual<<ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk, TryType: ~const Try> + ~const Destruct>
    {
        let mut remainder = ArrayBuffer::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner().into_inner().rev(), &mut remainder);
        bulk.try_for_each(f)?;
        Try::from_output(remainder.into_bulk())
    }

    #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
    pub const fn collect_with_remainder<C, A>(self) -> (C, <util::ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk)
    where
        Self: Sized,
        I: ~const Bulk<Item: ~const Destruct> + ~const DoubleEndedBulk,
        C: ~const FromBulk<A>,
        A: CollectionAdapter<Elem = [I::Item; N]> + for<'a> ~const CollectionStrategy<ArrayChunksWithRemainder::<'a, Rev<I>, N, true>, C> + ?Sized,
        util::ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner().into_inner().rev(), &mut remainder);
        let collection = bulk.collect();
        (
            collection,
            remainder.into_bulk()
        )
    }
}

impl<I, const N: usize> IntoIterator for ArrayChunks<I, N>
where
    I: Bulk
{
    type Item = [I::Item; N];
    type IntoIter = core::iter::ArrayChunks<I::IntoIter, N>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self {bulk} = self;
        bulk.into_iter().array_chunks()
    }
}
impl<I, const N: usize> const Bulk for ArrayChunks<I, N>
where
    I: ~const Bulk<Item: ~const Destruct>,
{
    type MinLength = length::Div<I::MinLength, [(); N]>;
    type MaxLength = length::Div<I::MaxLength, [(); N]>;

    #[inline]
    fn len(&self) -> usize
    {
        let Self {bulk} = self;
        bulk.len()/N
    }
    
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let mut remainder = ArrayBuffer::<_, _, false>::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner(), &mut remainder);
        bulk.for_each(f);
    }
    
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let mut remainder = ArrayBuffer::<_, _, false>::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner(), &mut remainder);
        bulk.try_for_each(f)?;
        Try::from_output(())
    }
}
impl<I, const N: usize> const DoubleEndedBulk for ArrayChunks<I, N>
where
    I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let mut remainder = ArrayBuffer::<_, _, true>::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner().rev(), &mut remainder);
        bulk.for_each(f);
    }
    
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let mut remainder = ArrayBuffer::<_, _, true>::new();
        let bulk = ArrayChunksWithRemainder::new(self.into_inner().rev(), &mut remainder);
        bulk.try_for_each(f)?;
        Try::from_output(())
    }
}
impl<I, const N: usize, L> const SplitBulk<L> for ArrayChunks<I, N>
where
    I: ~const SplitBulk<length::value::SaturatingMul<L, [(); N]>, Item: ~const Destruct, Left: ~const Bulk, Right: ~const Bulk>,
    L: LengthValue
{
    type Left = ArrayChunks<I::Left, N>;
    type Right = ArrayChunks<I::Right, N>;

    fn split_at(Self { bulk }: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let (left, right) = bulk.split_at(length::value::saturating_mul(n, [(); N]));
        (
            left.array_chunks(),
            right.array_chunks()
        )
    }
}
impl<'a, I, const N: usize> const RandomAccessBulk<'a> for ArrayChunks<I, N>
where
    I: ~const RandomAccessBulk<'a, Item: ~const Destruct>
{
    type ItemRef = [I::ItemRef; N];
    type EachRef = ArrayChunks<I::EachRef, N>;

    fn each_ref(Self { bulk }: &'a Self) -> Self::EachRef
    {
        bulk.each_ref().array_chunks()
    }
}
impl<'a, I, const N: usize> const InplaceBulk<'a> for ArrayChunks<I, N>
where
    I: ~const InplaceBulk<'a, Item: ~const Destruct>
{
    type ItemMut = [I::ItemMut; N];
    type EachMut = ArrayChunks<I::EachMut, N>;

    fn each_mut(Self { bulk }: &'a mut Self) -> Self::EachMut
    {
        bulk.each_mut().array_chunks()
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let b = const {
            let a = [1, 2, 3, 4, 5, 6];
            a.into_bulk().array_chunks::<2>().rev().enumerate().collect::<[_; _], _>()
        };

        println!("{b:?}");

        let c = b.into_bulk()
            .map(|(_, b)| b.into_bulk()
                .map(|b: u32| b.checked_sub(3))
                .collect::<Option<[_; _]>, [_; _]>()
            ).collect::<[_; _], _>();

        println!("{c:?}");

        let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let (a, b) = a.into_bulk().array_chunks::<4>().split_at(1);
        let (a, r) = a.collect_with_remainder::<Vec<_>, _>();
        assert!(r.is_empty());

        let (b, r) = b.collect_with_remainder::<Vec<_>, _>();
        let r = r.collect::<Vec<_>, _>();

        println!("a = {a:?}");
        println!("b = {b:?}");
        println!("r = {r:?}");
    }

    #[test]
    fn test_random_access()
    {
        let a = [1, 2, 3, 4, 5, 6];

        let mut b = a.into_bulk().array_chunks::<2>();

        for [e1, e2] in b.each_mut()
        {
            core::mem::swap(e1, e2);
        }

        let a = b.flatten().collect_array();

        println!("{a:?}")
    }
}