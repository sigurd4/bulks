use core::{marker::Destruct, ops::Try};

use array_trait::length::{self, LengthValue};

use crate::{Bulk, IntoBulk, SplitBulk};

pub mod option
{
    use array_trait::length::Length;

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

    pub(crate) trait MaybeLength = Length<Max<[(); 1]> = [(); 1], Elem = ()>;
}

macro_rules! impl_option {
    (
        impl $bulk:ident<$($a:lifetime,)? $t:ident>; for $item:ty; in $option:ty; $($mut:ident)?
        {
            fn first($self_first:ident) -> _
            $first:block
        }
    ) => {
        const impl<$($a,)? $t> IntoIterator for option::$bulk<$($a,)? $t>
        where
            $option: ~const IntoIterator
        {
            type Item = <$option as IntoIterator>::Item;
            type IntoIter = <$option as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter
            {
                self.option.into_iter()
            }
        }
        const impl<$($a,)? $t> IntoBulk for $option
        {
            type IntoBulk = option::$bulk<$($a,)? $t>;

            fn into_bulk(self) -> Self::IntoBulk
            {
                option::$bulk {
                    option: self
                }
            }
        }
        const impl<$($a,)? $t> Bulk for option::$bulk<$($a,)? $t>
        {
            type MinLength = [(); 0];
            type MaxLength = [(); 1];

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
                L: LengthValue
            {
                if length::value::len(n) == 0
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
const impl<T, L> SplitBulk<L> for option::IntoBulk<T>
where
    L: LengthValue
{
    type Left = Self;
    type Right = Self;

    fn split_at(bulk: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let empty = Self {
            option: None
        };

        if length::value::eq(n, [(); 0])
        {
            (empty, bulk)
        }
        else
        {
            (bulk, empty)
        }
    }
}
impl_option!(
    impl IntoBulk<T>; for T; in Option<T>; mut
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
    impl BulkMut<'a, T>; for &'a mut T; in &'a mut Option<T>; mut
    {
        fn first(self) -> _
        {
            self.option.as_mut()
        }
    }
);