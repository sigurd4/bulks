use core::{marker::Destruct, ops::Try};

use crate::{Bulk, util::LengthSpec};

pub mod option
{
    pub struct IntoBulk<T>
    {
        pub(super) option: Option<T>
    }

    pub struct Bulk<'a, T>
    {
        pub(super) option: &'a Option<T>
    }

    pub struct BulkMut<'a, T>
    {
        pub(super) option: &'a mut Option<T>
    }
}

macro_rules! impl_option {
    (
        impl $bulk:ident<$($a:lifetime,)? $t:ident>; for $item:ty; in $option:ty;
        {
            fn first($self_first:ident) -> _
            $first:block
        }
    ) => {
        impl<$($a,)? $t> IntoIterator for option::$bulk<$($a,)? $t>
        {
            type IntoIter = <$option as IntoIterator>::IntoIter;
            type Item = $item;

            fn into_iter(self) -> Self::IntoIter
            {
                self.option.into_iter()
            }
        }

        impl<$($a,)? $t> const Bulk for option::$bulk<$($a,)? $t>
        {
            type MinLength<U> = [U; 0];
            type MaxLength<U> = [U; 1];

            fn len(&self) -> usize
            {
                self.option.is_some() as usize
            }
            fn is_empty(&self) -> bool
            {
                self.option.is_none()
            }

            fn for_each<F>(self, mut f: F)
            where
                Self: Sized,
                F: ~const FnMut(Self::Item) + ~const Destruct
            {
                if let Some(x) = self.option
                {
                    f(x)
                }
            }
            fn try_for_each<F, R>(self, mut f: F) -> R
            where
                Self: Sized,
                Self::Item: ~const Destruct,
                F: ~const FnMut(Self::Item) -> R + ~const Destruct,
                R: ~const Try<Output = (), Residual: ~const Destruct>
            {
                if let Some(x) = self.option
                {
                    f(x)?
                }
                R::from_output(())
            }
                    
            fn first($self_first) -> Option<Self::Item>
            where
                Self::Item: ~const Destruct,
                Self: Sized
            $first
            fn last(self) -> Option<Self::Item>
            where
                Self::Item: ~const Destruct,
                Self: Sized
            {
                self.first()
            }
            fn nth<L>(self, n: L) -> Option<Self::Item>
            where
                Self: Sized,
                Self::Item: ~const Destruct,
                L: ~const LengthSpec
            {
                if n.len_metadata() == 0
                {
                    self.first()
                }
                else
                {
                    None
                }
            }
            
            fn reduce<F>(self, _f: F) -> Option<Self::Item>
            where 
                Self: Sized,
                Self::Item: ~const Destruct,
                F: ~const FnMut(Self::Item, Self::Item) -> Self::Item + ~const Destruct
            {
                self.first()
            }
        }
    };
}
impl_option!(
    impl IntoBulk<T>; for T; in Option<T>;
    {
        fn first(self) -> _
        {
            self.option
        }
    }
);
impl_option!(
    impl Bulk<'a, T>; for &'a T; in &'a Option<T>;
    {
        fn first(self) -> _
        {
            self.option.as_ref()
        }
    }
);
impl_option!(
    impl BulkMut<'a, T>; for &'a mut T; in &'a mut Option<T>;
    {
        fn first(self) -> _
        {
            self.option.as_mut()
        }
    }
);