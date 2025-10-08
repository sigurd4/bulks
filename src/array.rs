use crate::{Bulk, Copied, IntoBulk, LimitToBulk, StaticBulk, StaticCopiedSpec, StaticMapSpec, StaticRevSpec};

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
            fn collect_array($self_collect_array:ident) -> Self::Array
            $collect_array:block
        }
    ) => {
        impl<$($a,)? $t, const $n: usize> IntoIterator for array::$bulk<$($a,)? $t, $n>
        {
            type Item = $item;
            type IntoIter = <$array as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter
            {
                let Self {array} = self;
                array.into_iter()
            }
        }
        impl<$($a,)? $t, const $n: usize> LimitToBulk for array::$bulk<$($a,)? $t, $n>
        {
            
        }
        impl<$($a,)? $t, const $n: usize> IntoBulk for $array
        {
            type IntoBulk = array::$bulk<$($a,)? $t, $n>;
            
            fn into_bulk(self) -> Self::IntoBulk
            {
                array::$bulk {
                    array: self
                }
            }
        }
        impl<$($a,)? $t, const $n: usize> Bulk for array::$bulk<$($a,)? $t, $n>
        {
            fn len(&self) -> usize
            {
                $n
            }
        }
        impl<$($a,)? $t, const $n: usize> StaticBulk for array::$bulk<$($a,)? $t, $n>
        {
            type Array = [$item; N];

            fn collect_array($self_collect_array) -> Self::Array
            $collect_array
        }
    };
}
impl_bulk!(
    impl IntoBulk<T, const N: usize>; for T; in [T; N];
    {
        fn collect_array(self) -> Self::Array
        {
            let Self {array} = self;
            array
        }
    }
);
impl_bulk!(
    impl Bulk<'a, T, const N: usize>; for &'a T; in &'a [T; N];
    {
        fn collect_array(self) -> Self::Array
        {
            let Self {array} = self;
            array.each_ref()
        }
    }
);
impl_bulk!(
    impl BulkMut<'a, T, const N: usize>; for &'a mut T; in &'a mut [T; N];
    {
        fn collect_array(self) -> Self::Array
        {
            let Self {array} = self;
            array.each_mut()
        }
    }
);

impl<T, const N: usize> StaticMapSpec<N> for array::IntoBulk<T, N>
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
}

#[cfg(test)]
mod test
{
    use crate::{AsBulk, Bulk};

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3];
        let b: [_; _] = a.bulk().copied().rev().map(|x| 4 - x).collect::<[_; _], _>();
        println!("{:?}", b)
    }
}