use core::borrow::{Borrow, BorrowMut};
use core::{marker::Destruct, ops::Residual};
use core::ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Try};

use array_trait::AsSlice;

use crate::{AsBulk, Bulk, IntoBulk};

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
    #[cfg(feature = "alloc")]
    #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
    fn collect_nearest(self) -> Self::Nearest
    where
        Self: Sized;

    /// Fallibly collects into an array if possible, otherwise a vector
    #[cfg(feature = "alloc")]
    #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
    fn try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::TryNearest>>::TryType
    where
        Self: Sized,
        <Self as IntoIterator>::Item: ~const Try + ~const Destruct,
        <<Self as IntoIterator>::Item as Try>::Output: ~const Destruct,
        <<Self as IntoIterator>::Item as Try>::Residual: ~const Residual<Self::TryNearest> + ~const Residual<()> + ~const Destruct;
}

impl<I> CollectNearest for I
where
    I: Bulk + private::_CollectNearest<<I as Bulk>::Length>
{
    type Nearest = I::_Nearest;
    type TryNearest = I::_TryNearest
    where
        Self::Item: Try;

    fn collect_nearest(self) -> Self::Nearest
    {
        self._collect_nearest()
    }
    fn try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::TryNearest>>::TryType
    where
        <Self as IntoIterator>::Item: Try,
        <<Self as IntoIterator>::Item as Try>::Residual: Residual<Self::TryNearest> + Residual<()>
    {
        self._try_collect_nearest()
    }
}

mod private
{
    use core::{marker::Destruct, ops::{Try, Residual}};

    use array_trait::length::Length;

    use crate::{Bulk, Collection, StaticBulk};

    pub const trait _CollectNearest<L>: ~const Bulk
    where
        L: Length<Elem = ()> + ?Sized
    {
        #[allow(private_bounds)]
        type _Nearest: ~const Collection<Self::Item>;
        #[allow(private_bounds)]
        type _TryNearest: ~const Collection<<Self::Item as Try>::Output>
        where
            Self::Item: ~const Try;

        /// Collects into an array if possible, otherwise a vector
        #[cfg(feature = "alloc")]
        #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
        fn _collect_nearest(self) -> Self::_Nearest
        where
            Self: Sized;

        /// Fallibly collects into an array if possible, otherwise a vector
        #[cfg(feature = "alloc")]
        #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
        fn _try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::_TryNearest>>::TryType
        where
            Self: Sized,
            <Self as IntoIterator>::Item: ~const Try + ~const Destruct,
            <<Self as IntoIterator>::Item as Try>::Output: ~const Destruct,
            <<Self as IntoIterator>::Item as Try>::Residual: ~const Residual<Self::_TryNearest> + ~const Residual<()> + ~const Destruct;
    }

    #[cfg(feature = "alloc")]
    impl<I> _CollectNearest<[()]> for I
    where
        I: Bulk
    {
        type _Nearest = alloc::vec::Vec<I::Item>;
        type _TryNearest = alloc::vec::Vec<<I::Item as Try>::Output>
        where
            Self::Item: Try;

        fn _collect_nearest(self) -> Self::_Nearest
        {
            self.collect::<Self::_Nearest, _>()
        }
        fn _try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::_TryNearest>>::TryType
        where
            <Self as IntoIterator>::Item: Try,
            <<Self as IntoIterator>::Item as Try>::Residual: Residual<Self::_TryNearest> + Residual<()>
        {
            self.try_collect::<Self::_TryNearest, _>()
        }
    }
    impl<I, const N: usize> const _CollectNearest<[(); N]> for I
    where
        I: ~const Bulk + StaticBulk<Array<()> = [(); N]>
    {
        type _Nearest = I::Array<I::Item>;
        type _TryNearest = I::Array<<I::Item as Try>::Output>
        where
            Self::Item: Try;

        fn _collect_nearest(self) -> Self::_Nearest
        where
            Self: Sized
        {
            self.collect_array()
        }
        fn _try_collect_nearest(self) -> <<Self::Item as Try>::Residual as Residual<Self::_TryNearest>>::TryType
        where
            Self: Sized,
            <Self as IntoIterator>::Item: ~const Try + ~const Destruct,
            <<Self as IntoIterator>::Item as Try>::Output: ~const Destruct,
            <<Self as IntoIterator>::Item as Try>::Residual: ~const Residual<Self::_TryNearest> + ~const Residual<()> + ~const Destruct
        {
            self.try_collect_array()
        }
    }
}