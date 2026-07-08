use core::borrow::{Borrow, BorrowMut};
use core::marker::Destruct;
use core::ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Try};

use array_trait::AsSlice;
use array_trait::length::Length;

use crate::{AsBulk, Bulk,  CollectionAdapter, CollectionStrategy, FromBulk, IntoBulk, TryCollectionStrategy};

pub(crate) const trait Collection<T> = ~const IntoBulk<Item = T, IntoBulk: ~const Bulk<Item = T>>
    + ~const AsBulk
    + ~const AsSlice<Elem = T>
    + ~const AsRef<[T]>
    + ~const AsMut<[T]>
    + ~const Borrow<[T]>
    + ~const BorrowMut<[T]>
    + ~const IndexMut<usize, Output = <[T] as Index<usize>>::Output>
    + ~const IndexMut<Range<usize>, Output = <[T] as Index<Range<usize>>>::Output>
    + ~const IndexMut<RangeInclusive<usize>, Output = <[T] as Index<RangeInclusive<usize>>>::Output>
    + ~const IndexMut<RangeFrom<usize>, Output = <[T] as Index<RangeFrom<usize>>>::Output>
    + ~const IndexMut<RangeTo<usize>, Output = <[T] as Index<RangeTo<usize>>>::Output>
    + ~const IndexMut<RangeToInclusive<usize>, Output = <[T] as Index<RangeToInclusive<usize>>>::Output>
    + ~const IndexMut<RangeFull, Output = <[T] as Index<RangeFull>>::Output>;

#[rustc_on_unimplemented(
    message = "an array cannot be collected from dynamically sized bulk `{Self}`",
    label = "an array cannot be collected from bulk"
)]
pub const trait Nearest: CollectionAdapter<Elem = ()>
{
    #[allow(private_bounds)]
    type Nearest<B>: ~const Collection<B::Item> + ~const FromBulk<Self::NearestStrategy<B>>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    #[allow(private_bounds)]
    type TryNearest<B>: ~const Collection<<B::Item as Try>::Output> + ~const FromBulk<Self::TryNearestStrategy<B>>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;

    #[allow(private_bounds)]
    type NearestStrategy<B>: ~const CollectionStrategy<B::MinLength, B::MaxLength, Self::Nearest<B>> + CollectionAdapter<Elem = B::Item>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    #[allow(private_bounds)]
    type TryNearestStrategy<B>: ~const TryCollectionStrategy<B::MinLength, B::MaxLength, Self::TryNearest<B>> + CollectionAdapter<Elem = <B::Item as Try>::Output>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;
}

#[cfg(feature = "alloc")]
impl<I> Nearest for I
where
    I: Bulk
{
    type Adapter<T> = [T];
    default type Nearest = alloc::vec::Vec<I::Item>;
    default type TryNearest = alloc::vec::Vec<<I::Item as Try>::Output>
    where
        Self::Item: Try;
}
const impl<const N: usize> Nearest for [(); N]
{
    type Nearest<B> = [B::Item; N]
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    type TryNearest<B> = [<B::Item as Try>::Output; N]
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;

    type NearestStrategy<B> = [B::Item; N]
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    type TryNearestStrategy<B> = [<B::Item as Try>::Output; N]
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;
}