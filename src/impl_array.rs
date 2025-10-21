use core::{marker::Destruct, ops::Try};

use crate::{util::{self, LengthSpec}, Bulk, Guard, IntoBulk, StaticBulk, DoubleEndedBulk};

pub mod array
{
    #[derive(Clone, Debug)]
    pub struct IntoBulk<T, const N: usize>
    {
        pub(super) array: [T; N]
    }

    #[derive(Clone, Debug)]
    pub struct Bulk<'a, T, const N: usize>
    {
        pub(super) array: &'a [T; N]
    }

    #[derive(Debug)]
    pub struct BulkMut<'a, T, const N: usize>
    {
        pub(super) array: &'a mut [T; N]
    }
}

macro_rules! impl_bulk {
    (
        impl $bulk:ident<$($a:lifetime,)? $t:ident, const $n:ident: usize>; for $item:ty; in $array:ty;
        {
            fn for_each($self_for_each:ident, $f_for_each:ident) -> _
            $for_each:block

            fn try_for_each($self_try_for_each:ident, $f_try_for_each:ident) -> _
            $try_for_each:block

            fn rev_for_each($self_rev_for_each:ident, $f_rev_for_each:ident) -> _
            $rev_for_each:block

            fn try_rev_for_each($self_try_rev_for_each:ident, $f_try_rev_for_each:ident) -> _
            $try_rev_for_each:block

            fn collect_array($self_collect_array:ident) -> _
            $collect_array:block

            $(fn nth($self_nth:ident, $n_nth:ident) -> _
            $nth:block)?
        }
    ) => {
        impl<$($a,)? $t, const $n: usize> IntoIterator for array::$bulk<$($a,)? $t, $n>
        {
            type Item = $item;
            type IntoIter = <$array as IntoIterator>::IntoIter;

            #[inline]
            fn into_iter(self) -> Self::IntoIter
            {
                let Self {array} = self;
                array.into_iter()
            }
        }
        impl<$($a,)? $t, const $n: usize> const IntoBulk for $array
        {
            type IntoBulk = array::$bulk<$($a,)? $t, $n>;
            
            #[inline]
            fn into_bulk(self) -> Self::IntoBulk
            {
                array::$bulk {
                    array: self
                }
            }
        }
        impl<$($a,)? $t, const $n: usize> const Bulk for array::$bulk<$($a,)? $t, $n>
        {
            #[inline]
            fn len(&self) -> usize
            {
                $n
            }

            fn first(self) -> Option<Self::Item>
            where
                Self::Item: ~const Destruct,
                Self: Sized
            {
                self.nth([(); 0])
            }
            
            $(fn nth<L>($self_nth, $n_nth: L) -> Option<Self::Item>
            where
                Self::Item: ~const Destruct,
                Self: Sized,
                L: ~const LengthSpec
            $nth)?

            #[inline]
            fn collect_array($self_collect_array) -> <Self as StaticBulk>::Array<Self::Item>
            $collect_array

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
        impl<$($a,)? $t, const $n: usize> const DoubleEndedBulk for array::$bulk<$($a,)? $t, $n>
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
        impl<$($a,)? $t, const $n: usize> StaticBulk for array::$bulk<$($a,)? $t, $n>
        {
            type Array<U> = [U; $n];
        }
    };
}
impl_bulk!(
    impl IntoBulk<T, const N: usize>; for T; in [T; N];
    {
        fn for_each(self, f) -> _
        {
            let Self { array } = self;
            let mut src_array = util::new_init_array(array);
            let mut src_guard = Guard { array_mut: &mut src_array, initialized: 0..N };

            while src_guard.initialized.start < src_guard.initialized.end
            {
                unsafe {
                    f(src_guard.pop_front_unchecked())
                }
            }

            core::mem::forget(src_guard);
        }

        fn try_for_each(self, f) -> _
        {
            let Self { array } = self;
            let mut src_array = util::new_init_array(array);
            let mut src_guard = Guard { array_mut: &mut src_array, initialized: 0..N };

            while src_guard.initialized.start < src_guard.initialized.end
            {
                unsafe {
                    f(src_guard.pop_front_unchecked())?
                }
            }

            core::mem::forget(src_guard);
            R::from_output(())
        }

        fn rev_for_each(self, f) -> _
        {
            let Self { array } = self;
            let mut src_array = util::new_init_array(array);
            let mut src_guard = Guard { array_mut: &mut src_array, initialized: 0..N };

            while src_guard.initialized.start < src_guard.initialized.end
            {
                unsafe {
                    f(src_guard.pop_back_unchecked())
                }
            }

            core::mem::forget(src_guard);
        }

        fn try_rev_for_each(self, f) -> _
        {
            let Self { array } = self;
            let mut src_array = util::new_init_array(array);
            let mut src_guard = Guard { array_mut: &mut src_array, initialized: 0..N };

            while src_guard.initialized.start < src_guard.initialized.end
            {
                unsafe {
                    f(src_guard.pop_back_unchecked())?
                }
            }

            core::mem::forget(src_guard);
            R::from_output(())
        }

        fn collect_array(self) -> _
        {
            let Self {array} = self;
            array
        }
    }
);
impl_bulk!(
    impl Bulk<'a, T, const N: usize>; for &'a T; in &'a [T; N];
    {
        fn for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = 0;
            while n < N
            {
                f(&array[n]);
                n += 1;
            }
        }

        fn try_for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = 0;
            while n < N
            {
                f(&array[n])?;
                n += 1;
            }
            R::from_output(())
        }

        fn rev_for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = N;
            while n > 0
            {
                n -= 1;
                f(&array[n]);
            }
        }

        fn try_rev_for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = N;
            while n > 0
            {
                n -= 1;
                f(&array[n])?;
            }
            R::from_output(())
        }

        fn collect_array(self) -> _
        {
            let Self {array} = self;
            array.each_ref()
        }

        fn nth(self, n) -> _
        {
            let Self {array} = self;
            array.get(n.len_metadata())
        }
    }
);
impl_bulk!(
    impl BulkMut<'a, T, const N: usize>; for &'a mut T; in &'a mut [T; N];
    {
        fn for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = 0;
            while n < N
            {
                f(unsafe {&mut *(&mut array[n] as *mut _)});
                n += 1;
            }
        }

        fn try_for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = 0;
            while n < N
            {
                f(unsafe {&mut *(&mut array[n] as *mut _)})?;
                n += 1;
            }
            R::from_output(())
        }

        fn rev_for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = N;
            while n > 0
            {
                n -= 1;
                f(unsafe {&mut *(&mut array[n] as *mut _)});
            }
        }

        fn try_rev_for_each(self, f) -> _
        {
            let Self {array} = self;
            let mut n = N;
            while n > 0
            {
                n -= 1;
                f(unsafe {&mut *(&mut array[n] as *mut _)})?;
            }
            R::from_output(())
        }

        fn collect_array(self) -> _
        {
            let Self {array} = self;
            array.each_mut()
        }

        fn nth(self, n) -> _
        {
            let Self {array} = self;
            array.get_mut(n.len_metadata())
        }
    }
);

/*impl<T, const N: usize> StaticMapSpec<N> for array::IntoBulk<T, N>
{
    fn map_collect_array<U>(self, f: impl FnMut(Self::Item) -> U) -> [U; N]
    {
        let Self {array} = self;
        array.map(f)
    }
}
impl<T, const N: usize> StaticRevSpec<N> for array::IntoBulk<T, N>
{
    fn rev_collect_array(self) -> [T; N]
    {
        let Self {mut array} = self;
        array.reverse();
        array
    }
}

impl<'a, T, const N: usize> StaticMapSpec<N> for Copied<array::Bulk<'a, T, N>>
where
    T: Copy + 'a
{
    fn map_collect_array<U>(self, f: impl FnMut(Self::Item) -> U) -> [U; N]
    {
        self.into_inner()
            .copied_collect_array()
            .into_bulk()
            .map(f)
            .collect()
    }
}
impl<'a, T, const N: usize> StaticRevSpec<N> for Copied<array::Bulk<'a, T, N>>
where
    T: Copy + 'a
{
    fn rev_collect_array(self) -> [T; N]
    {
        self.into_inner()
            .copied_collect_array()
            .into_bulk()
            .rev()
            .collect()
    }
}

impl<'a, T, const N: usize> StaticCopiedSpec<N> for array::Bulk<'a, T, N>
where
    T: Copy + 'a
{
    fn copied_collect_array(self) -> [T; N]
    {
        let Self {array} = self;
        *array
    }
}*/

#[cfg(test)]
mod test
{
    use crate::{AsBulk, Bulk};

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3];
        
        let b = a.bulk().copied().rev().map(|x| 4 - x).collect::<[_; _], _>();

        println!("{:?}", b)
    }
}