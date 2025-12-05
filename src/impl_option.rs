use core::{marker::Destruct, ops::Try};

use array_trait::length::{self, LengthValue};

use crate::{AsBulk, Bulk, IntoBulk, RandomAccessBulkMut, RandomAccessBulk};

pub mod option
{
    use array_trait::length::Length;

    use crate::{CollectionAdapter, FromBulk, StaticBulk};

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

    pub trait MaybeLength = Length<Max<[(); 1]> = [(); 1], Elem = ()>;

    pub trait Maybe: CollectionAdapter<Elem = Self::Item> + crate::IntoBulk<IntoBulk: MaybeBulk> + FromBulk<Self>
    {
        type Not: Maybe<Not = Self>;
    }
    impl<T> Maybe for Option<T>
    {
        type Not = Option<T>;
    }
    impl<T> Maybe for [T; 0]
    {
        type Not = [T; 1];
    }
    impl<T> Maybe for [T; 1]
    {
        type Not = [T; 0];
    }

    pub const trait MaybeBulk: ~const crate::Bulk<MaxLength: MaybeLength, MinLength: MaybeLength>
    {
        type Maybe: Maybe;
    }

    impl<A, T> const MaybeBulk for T
    where
        T: ~const crate::Bulk<Item = A, MaxLength: MaybeLength, MinLength: MaybeLength>
    {
        default type Maybe = Option<A>;
    }
    impl<A, T, const N: usize> const MaybeBulk for T
    where
        T: ~const crate::Bulk<Item = A> + StaticBulk<Array<()> = [(); N], Array<A> = [A; N]>,
        [A; N]: Maybe<Item = A>,
        [(); N]: MaybeLength
    {
        type Maybe = [A; N];
    }
}

macro_rules! impl_option {
    (
        impl $bulk:ident<$($a:lifetime,)? $t:ident>; for $item:ty; in $option:ty; $($mut:ident)?
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
        impl<$($a,)? $t> const IntoBulk for $option
        {
            type IntoBulk = option::$bulk<$($a,)? $t>;

            fn into_bulk(self) -> Self::IntoBulk
            {
                option::$bulk {
                    option: self
                }
            }
        }
        impl<$($a,)? $t> const Bulk for option::$bulk<$($a,)? $t>
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
        impl<$($a,)? 'b, T> RandomAccessBulk<'b> for option::$bulk<$($a,)? $t>
        where
            Self: 'b
        {
            type ItemRef = &'b T;
            type EachRef = option::Bulk<'b, T>;

            fn each_ref(bulk: &'b Self) -> Self::EachRef
            {
                (&bulk.option as &Option<T>).bulk()
            }
        }
        impl_option!(@extra impl $bulk<$($a,)? $t>; for $item; in $option; $($mut)?);
    };
    (
        @extra impl $bulk:ident<$($a:lifetime,)? $t:ident>; for $item:ty; in $option:ty; $mut:ident
    ) => {
        impl<$($a,)? 'b, T> RandomAccessBulkMut<'b> for option::$bulk<$($a,)? T>
        where
            Self: 'b
        {
            type ItemMut = &'b mut T;
            type EachMut = option::BulkMut<'b, T>;

            fn each_mut(bulk: &'b mut Self) -> Self::EachMut
            {
                (&mut bulk.option as &mut Option<T>).bulk_mut()
            }
        }
    };
    (
        @extra impl $bulk:ident<$($a:lifetime,)? $t:ident>; for $item:ty; in $option:ty;
    ) => {
        
    };
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