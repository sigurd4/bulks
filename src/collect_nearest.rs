use core::borrow::{Borrow, BorrowMut};
use core::marker::Destruct;
use core::ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Try};

use array_trait::AsSlice;
use array_trait::length::Length;

use crate::{AsBulk, Bulk,  CollectionAdapter, CollectionStrategy, FromBulk, IntoBulk, TryCollectionStrategy};

pub(crate) const trait Collection<T, L: Nearest + ?Sized> = ~const IntoBulk<Item = T, IntoBulk: ~const Bulk<Item = T, /*MinLength: Length<Intersect<L> = L>,*/ MaxLength = L>>
    + ~const FromBulk<L::NearestStrategy<T>>
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
    message = "an array cannot be collected from dynamically sized bulk of length `{Self}`",
    label = "an array cannot be collected from bulk"
)]
pub const trait Nearest: Length<Elem = ()>
{
    #[allow(private_bounds)]
    type Nearest<T>: ~const Collection<T, Self>
        + ~const FromBulk<Self::NearestStrategy<T>>
    where
        T: ~const Destruct;
    #[allow(private_bounds)]
    type TryNearest<T>: ~const Collection<<T as Try>::Output, Self>
        + ~const FromBulk<Self::TryNearestStrategy<T>>
        + ~const FromBulk<Self::NearestStrategy<<T as Try>::Output>>
        + const From<Self::Nearest<<T as Try>::Output>> + const Into<Self::Nearest<<T as Try>::Output>>
    where
    where
        T: ~const Try<Output: ~const Destruct>;

    #[allow(private_bounds)]
    type NearestFrom<B>: ~const Collection<B::Item, Self>
        + ~const FromBulk<Self::NearestStrategyFrom<B>>
        + ~const FromBulk<Self::NearestStrategy<B::Item>>
        + const From<Self::Nearest<B::Item>> + const Into<Self::Nearest<B::Item>>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    #[allow(private_bounds)]
    type TryNearestFrom<B>: ~const Collection<<B::Item as Try>::Output, Self>
        + ~const FromBulk<Self::TryNearestStrategyFrom<B>>
        + ~const FromBulk<Self::TryNearestStrategy<B::Item>>
        + ~const FromBulk<Self::NearestStrategy<<B::Item as Try>::Output>>
        + const From<Self::TryNearest<B::Item>> + const Into<Self::TryNearest<B::Item>>
        + const From<Self::Nearest<<B::Item as Try>::Output>> + const Into<Self::Nearest<<B::Item as Try>::Output>>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;

    #[allow(private_bounds)]
    type NearestStrategy<T>: ~const CollectionStrategy<Self, Self, Self::Nearest<T>>
        + CollectionAdapter<Elem = T>
        + ?Sized;
    #[allow(private_bounds)]
    type TryNearestStrategy<T>: ~const TryCollectionStrategy<Self, Self, Self::TryNearest<T>>
        + ~const TryCollectionStrategy<Self, Self, Self::Nearest<<T as Try>::Output>>
        + ~const CollectionStrategy<Self, Self, Self::TryNearest<T>>
        + ~const CollectionStrategy<Self, Self, Self::Nearest<<T as Try>::Output>>
        + CollectionAdapter<Elem = <T as Try>::Output>
        + ?Sized
    where
        T: ~const Try<Output: ~const Destruct>;

    #[allow(private_bounds)]
    type NearestStrategyFrom<B>: ~const CollectionStrategy<B::MinLength, B::MaxLength, Self::NearestFrom<B>>
        + ~const CollectionStrategy<B::MinLength, B::MaxLength, Self::Nearest<B::Item>>
        + ~const CollectionStrategy<Self, Self, Self::NearestFrom<B>>
        + ~const CollectionStrategy<Self, Self, Self::Nearest<B::Item>>
        + CollectionAdapter<Elem = B::Item>
        + ?Sized
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    #[allow(private_bounds)]
    type TryNearestStrategyFrom<B>: ~const TryCollectionStrategy<B::MinLength, B::MaxLength, Self::TryNearestFrom<B>>
        + ~const TryCollectionStrategy<B::MinLength, B::MaxLength, Self::TryNearest<B::Item>>
        + ~const TryCollectionStrategy<B::MinLength, B::MaxLength, Self::Nearest<<B::Item as Try>::Output>>
        + ~const TryCollectionStrategy<Self, Self, Self::TryNearestFrom<B>>
        + ~const TryCollectionStrategy<Self, Self, Self::TryNearest<B::Item>>
        + ~const TryCollectionStrategy<Self, Self, Self::Nearest<<B::Item as Try>::Output>>
        + ~const CollectionStrategy<B::MinLength, B::MaxLength, Self::TryNearestFrom<B>>
        + ~const CollectionStrategy<B::MinLength, B::MaxLength, Self::TryNearest<B::Item>>
        + ~const CollectionStrategy<B::MinLength, B::MaxLength, Self::Nearest<<B::Item as Try>::Output>>
        + ~const CollectionStrategy<Self, Self, Self::TryNearestFrom<B>>
        + ~const CollectionStrategy<Self, Self, Self::TryNearest<B::Item>>
        + ~const CollectionStrategy<Self, Self, Self::Nearest<<B::Item as Try>::Output>>
        + CollectionAdapter<Elem = <B::Item as Try>::Output>
        + ?Sized
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;
}

#[cfg(feature = "alloc")]
impl Nearest for [()]
{
    type Nearest<T> = alloc::vec::Vec<T>;
    type TryNearest<T> = alloc::vec::Vec<<T as Try>::Output>
    where
        T: Try;

    type NearestStrategy<T> = [T];
    type TryNearestStrategy<T> = [<T as Try>::Output]
    where
        T: Try;

    type NearestFrom<B> = Self::Nearest<B::Item>
    where
        B: Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>>;
    type TryNearestFrom<B> = Self::TryNearest<B::Item>
    where
        B: Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: Try>;

    type NearestStrategyFrom<B> = Self::NearestStrategy<B::Item>
    where
        B: Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>>;
    type TryNearestStrategyFrom<B> = Self::TryNearestStrategy<B::Item>
    where
        B: Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: Try>;
}
const impl<const N: usize> Nearest for [(); N]
{
    type Nearest<T> = [T; N]
    where
        T: ~const Destruct;
    type TryNearest<T> = [<T as Try>::Output; N]
    where
        T: ~const Try<Output: ~const Destruct>;

    type NearestStrategy<T> = [T; N];
    type TryNearestStrategy<T> = [<T as Try>::Output; N]
    where
        T: ~const Try<Output: ~const Destruct>;

    type NearestFrom<B> = Self::Nearest<B::Item>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    type TryNearestFrom<B> = Self::TryNearest<B::Item>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;

    type NearestStrategyFrom<B> = Self::NearestStrategy<B::Item>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Destruct>;
    type TryNearestStrategyFrom<B> = Self::TryNearestStrategy<B::Item>
    where
        B: ~const Bulk<MinLength: Length<Intersect<B::MaxLength> = Self>, Item: ~const Try<Output: ~const Destruct>>;
}

pub trait CollectNearest = Bulk<MinLength: Length<Intersect<<Self as Bulk>::MaxLength>: Nearest>>;