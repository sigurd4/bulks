use core::{marker::Destruct, ops::Try};

use array_trait::length::{self, LengthValue};

use crate::{Bulk, AsBulk, InplaceBulk, IntoBulk, RandomAccessBulk, DoubleEndedBulk, SplitBulk};

pub mod slice
{
    pub struct Bulk<'a, T>
    where
        T: 'a
    {
        pub(super) slice: &'a [T]
    }

    pub struct BulkMut<'a, T>
    where
        T: 'a
    {
        pub(super) slice: &'a mut [T]
    }
}

macro_rules! impl_bulk {
    (
        impl $bulk:ident<$a:lifetime, $t:ident>; for $item:ty; in $slice:ty; $($mut:ident)?
        {
            fn for_each($self_for_each:ident, $f_for_each:ident) -> _
            $for_each:block

            fn try_for_each($self_try_for_each:ident, $f_try_for_each:ident) -> _
            $try_for_each:block

            fn rev_for_each($self_rev_for_each:ident, $f_rev_for_each:ident) -> _
            $rev_for_each:block

            fn try_rev_for_each($self_try_rev_for_each:ident, $f_try_rev_for_each:ident) -> _
            $try_rev_for_each:block

            fn split_at($self_split_at:ident, $n_split_at:ident) -> _
            $split_at:block

            fn nth($self_nth:ident, $n_nth:ident) -> _
            $nth:block
        }
    ) => {
        impl<$a, $t> IntoIterator for slice::$bulk<$a, $t>
        where
            $t: $a
        {
            type Item = $item;
            type IntoIter = <$slice as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter
            {
                self.slice.into_iter()
            }
        }
        impl<$a, $t> const IntoBulk for $slice
        where
            $t: $a
        {
            type IntoBulk = slice::$bulk<$a, $t>;
            
            fn into_bulk(self) -> Self::IntoBulk
            {
                slice::$bulk {
                    slice: self
                }
            }
        }
        impl<'a, T> const Bulk for slice::$bulk<'a, T>
        where
            T: 'a
        {
            fn len(&self) -> usize
            {
                self.slice.len()
            }
            fn is_empty(&self) -> bool
            {
                self.slice.is_empty()
            }

            fn first(self) -> Option<Self::Item>
            where
                Self::Item: ~const Destruct,
                Self: Sized
            {
                self.nth([(); 0])
            }
            fn nth<L>($self_nth, $n_nth: L) -> Option<Self::Item>
            where
                Self::Item: ~const Destruct,
                Self: Sized,
                L: LengthValue
            $nth

            fn get<'b, L>(&'b self, i: L) -> Option<&'b <Self as RandomAccessBulk>::ItemPointee>
            where
                L: LengthValue,
                Self: 'b
            {
                self.slice.get(length::value::len(i))
            }

            $(fn ${concat(get_, $mut)}<'b, L>(&'b mut self, i: L) -> Option<&'b mut <Self as RandomAccessBulk>::ItemPointee>
            where
                L: LengthValue,
                Self: 'b
            {
                self.slice.get_mut(length::value::len(i))
            })?

            #[inline]
            fn for_each<F>($self_for_each, mut $f_for_each: F)
            where
                F: ~const FnMut($item) + ~const Destruct
            $for_each

            #[inline]
            fn try_for_each<F, R>($self_try_for_each, mut $f_try_for_each: F) -> R
            where
                $item: ~const Destruct,
                F: ~const FnMut($item) -> R + ~const Destruct,
                R: ~const Try<Output = (), Residual: ~const Destruct>
            $try_for_each
        }
        impl<$a, $t> const DoubleEndedBulk for slice::$bulk<$a, $t>
        {
            #[inline]
            fn rev_for_each<F>($self_rev_for_each, mut $f_rev_for_each: F)
            where
                F: ~const FnMut($item) + ~const Destruct
            $rev_for_each

            #[inline]
            fn try_rev_for_each<F, R>($self_try_rev_for_each, mut $f_try_rev_for_each: F) -> R
            where
                $item: ~const Destruct,
                F: ~const FnMut($item) -> R + ~const Destruct,
                R: ~const Try<Output = (), Residual: ~const Destruct>
            $try_rev_for_each
        }
        impl<$a, T, L> SplitBulk<L> for slice::$bulk<$a, T>
        where
            L: LengthValue
        {
            type Left = Self;
            type Right = Self;

            #[track_caller]
            fn split_at($self_split_at: Self, $n_split_at: L) -> (Self::Left, Self::Right)
            where
                Self: Sized
            $split_at
        }
        impl<$a, T> const RandomAccessBulk for slice::$bulk<$a, T>
        {
            type ItemPointee = T;
            type EachRef<'b> = slice::Bulk<'b, T>
            where
                Self::ItemPointee: 'b,
                Self: 'b;

            fn each_ref<'b>(bulk: &'b Self) -> Self::EachRef<'b>
            where
                Self::ItemPointee: 'b,
                Self: 'b
            {
                (&bulk.slice as &[T]).bulk()
            }
        }
        impl_bulk!(@extra impl $bulk<$a, $t>; for $item; in $slice; $($mut)?);
    };
    (
        @extra impl $bulk:ident<$a:lifetime, $t:ident>; for $item:ty; in $slice:ty; $mut:ident
    ) => {
        impl<$a, T> const InplaceBulk for slice::$bulk<$a, T>
        {
            type EachMut<'b> = slice::BulkMut<'b, T>
            where
                Self::ItemPointee: 'b,
                Self: 'b;

            fn each_mut<'b>(bulk: &'b mut Self) -> Self::EachMut<'b>
            where
                Self::ItemPointee: 'b,
                Self: 'b
            {
                (&mut bulk.slice as &mut [T]).bulk_mut()
            }
        }
    };
    (
        @extra impl $bulk:ident<$a:lifetime, $t:ident>; for $item:ty; in $slice:ty;
    ) => {
        
    };
}
impl_bulk!(
    impl Bulk<'a, T>; for &'a T; in &'a [T];
    {
        fn for_each(self, f) -> _
        {
            let Self { slice } = self;
            let len = slice.len();
            let mut n = 0;
            while n < len
            {
                f(&slice[n]);
                n += 1;
            }
        }

        fn try_for_each(self, f) -> _
        {
            let Self { slice } = self;
            let len = slice.len();
            let mut n = 0;
            while n < len
            {
                f(&slice[n])?;
                n += 1;
            }
            R::from_output(())
        }

        fn rev_for_each(self, f) -> _
        {
            let Self { slice } = self;
            let mut n = slice.len();
            while n > 0
            {
                n -= 1;
                f(&slice[n]);
            }
        }

        fn try_rev_for_each(self, f) -> _
        {
            let Self { slice } = self;
            let mut n = slice.len();
            while n > 0
            {
                n -= 1;
                f(&slice[n])?;
            }
            R::from_output(())
        }

        fn split_at(bulk, n) -> _
        {
            let n = length::value::len(n);
            let Self { slice } = bulk;
            let (left, right) = slice.split_at(n);
            (left.into_bulk(), right.into_bulk())
        }

        fn nth(self, n) -> _
        {
            let Self { slice } = self;
            slice.get(length::value::len(n))
        }
    } 
);
impl_bulk!(
    impl BulkMut<'a, T>; for &'a mut T; in &'a mut [T]; mut
    {
        fn for_each(self, f) -> _
        {
            let Self { slice } = self;
            let len = slice.len();
            let mut n = 0;
            while n < len
            {
                f(unsafe {&mut *(&mut slice[n] as *mut _)});
                n += 1;
            }
        }

        fn try_for_each(self, f) -> _
        {
            let Self { slice } = self;
            let len = slice.len();
            let mut n = 0;
            while n < len
            {
                f(unsafe {&mut *(&mut slice[n] as *mut _)})?;
                n += 1;
            }
            R::from_output(())
        }

        fn rev_for_each(self, f) -> _
        {
            let Self { slice } = self;
            let mut n = slice.len();
            while n > 0
            {
                n -= 1;
                f(unsafe {&mut *(&mut slice[n] as *mut _)});
            }
        }

        fn try_rev_for_each(self, f) -> _
        {
            let Self { slice } = self;
            let mut n = slice.len();
            while n > 0
            {
                n -= 1;
                f(unsafe {&mut *(&mut slice[n] as *mut _)})?;
            }
            R::from_output(())
        }

        fn split_at(bulk, n) -> _
        {
            let n = length::value::len(n);
            let Self { slice } = bulk;
            let (left, right) = slice.split_at_mut(n);
            (left.into_bulk(), right.into_bulk())
        }

        fn nth(self, n) -> _
        {
            let Self { slice } = self;
            slice.get_mut(length::value::len(n))
        }
    } 
);