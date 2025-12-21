use core::borrow::{Borrow, BorrowMut};
use core::{marker::Destruct, ops::Residual};
use core::ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Try};

use array_trait::AsSlice;

use crate::{AsBulk, Bulk, IntoBulk, StaticBulk};

pub(crate) const trait Collection<T> = ~const IntoBulk<Item = T/*, IntoBulk: for<'a> ~const RandomAccessBulk<'a>*/>
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

pub const trait CollectNearest: ~const Bulk
{
    #[allow(private_bounds)]
    type Nearest: ~const Collection<Self::Item>;
    #[allow(private_bounds)]
    type TryNearest: ~const Collection<<Self::Item as Try>::Output>
    where
        Self::Item: ~const Try;

    /// Collects into an array if possible, otherwise a vector
    #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
    fn collect_nearest(self) -> Self::Nearest
    where
        Self: Sized;

    /// Fallibly collects into an array if possible, otherwise a vector
    #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
    fn try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::TryNearest>>::TryType
    where
        Self: Sized,
        <Self as IntoIterator>::Item: ~const Try + ~const Destruct,
        <<Self as IntoIterator>::Item as Try>::Output: ~const Destruct,
        <<Self as IntoIterator>::Item as Try>::Residual: ~const Residual<Self::TryNearest> + ~const Residual<()> + ~const Destruct;
}

#[cfg(feature = "alloc")]
impl<I> CollectNearest for I
where
    I: Bulk
{
    default type Nearest = alloc::vec::Vec<I::Item>;
    default type TryNearest = alloc::vec::Vec<<I::Item as Try>::Output>
    where
        Self::Item: Try;

    default fn collect_nearest(self) -> Self::Nearest
    {
        use array_trait::same::Same;

        self.collect::<alloc::vec::Vec<_>, _>().same().ok().unwrap()
    }
    default fn try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::TryNearest>>::TryType
    where
        <Self as IntoIterator>::Item: Try,
        <<Self as IntoIterator>::Item as Try>::Residual: Residual<Self::TryNearest> + Residual<()>
    {
        vec_spec::CollectVecSpec::<Self::TryNearest>::try_collect_vec(self)
    }
}
impl<I, const N: usize> const CollectNearest for I
where
    I: ~const Bulk + StaticBulk<Array<()> = [(); N]>
{
    type Nearest = I::Array<I::Item>;
    type TryNearest = I::Array<<I::Item as Try>::Output>
    where
        Self::Item: Try;

    fn collect_nearest(self) -> Self::Nearest
    where
        Self: Sized
    {
        self.collect_array()
    }
    fn try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::TryNearest>>::TryType
    where
        Self: Sized,
        <Self as IntoIterator>::Item: ~const Try + ~const Destruct,
        <<Self as IntoIterator>::Item as Try>::Output: ~const Destruct,
        <<Self as IntoIterator>::Item as Try>::Residual: ~const Residual<Self::TryNearest> + ~const Residual<()> + ~const Destruct
    {
        self.try_collect_array()
    }
}

#[cfg(feature = "alloc")]
mod vec_spec
{
    use core::ops::{Residual, Try};

    use array_trait::same::Same;

    use crate::Bulk;

    pub(super) trait CollectVecSpec<C>: Bulk
    {
        fn try_collect_vec(bulk: Self) -> <<Self::Item as Try>::Residual as Residual<C>>::TryType
        where
            <Self as IntoIterator>::Item: Try,
            <<Self as IntoIterator>::Item as Try>::Residual: Residual<C> + Residual<()>;
    }

    impl<I, C> CollectVecSpec<C> for I
    where
        I: Bulk
    {
        default fn try_collect_vec(bulk: Self) -> <<Self::Item as Try>::Residual as Residual<C>>::TryType
        where
            <Self as IntoIterator>::Item: Try,
            <<Self as IntoIterator>::Item as Try>::Residual: Residual<C> + Residual<()>
        {
            let mut vec = alloc::vec::Vec::with_capacity(bulk.len());
            for item in bulk
            {
                vec.push(item?);
            }
            Try::from_output(vec.same().ok().unwrap())
        }
    }
    impl<I> CollectVecSpec<alloc::vec::Vec<<I::Item as Try>::Output>> for I
    where
        I: Bulk<Item: Try>
    {
        fn try_collect_vec(bulk: Self) -> <<Self::Item as Try>::Residual as Residual<alloc::vec::Vec<<I::Item as Try>::Output>>>::TryType
        where
            <Self as IntoIterator>::Item: Try,
            <<Self as IntoIterator>::Item as Try>::Residual: Residual<alloc::vec::Vec<<I::Item as Try>::Output>> + Residual<()>
        {
            bulk.try_collect::<alloc::vec::Vec<_>, _>()
        }
    }
}