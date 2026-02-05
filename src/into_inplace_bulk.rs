use crate::{InplaceBulk, IntoBulk};

pub const trait IntoInplaceBulk: IntoBulk
{
    type IntoInplaceBulk: ~const InplaceBulk;

    fn into_inplace_bulk(self) -> Self::IntoInplaceBulk;
}

impl<T> const IntoInplaceBulk for T
where
    T: ~const private::_IntoInplaceBulk<_IntoInplaceBulk: ~const InplaceBulk>
{
    type IntoInplaceBulk = <T as private::_IntoInplaceBulk>::_IntoInplaceBulk;

    fn into_inplace_bulk(self) -> Self::IntoInplaceBulk
    {
        self._into_inplace_bulk()
    }
}

mod private
{
    use array_trait::same::Same;

    use crate::{Bulk, CollectNearest, InplaceBulk, IntoBulk};

    pub const trait _IntoInplaceBulk: IntoBulk
    {
        type _IntoInplaceBulk: ~const Bulk<Item = Self::Item>;

        fn _into_inplace_bulk(self) -> Self::_IntoInplaceBulk;
    }
    impl<T> const _IntoInplaceBulk for T
    where
        T: ~const IntoBulk
    {
        default type _IntoInplaceBulk = T::IntoBulk;

        default fn _into_inplace_bulk(self) -> Self::_IntoInplaceBulk
        {
            unsafe {
                self.into_bulk().same().unwrap_unchecked()
            }
        }
    }
    impl<T, B, N> const _IntoInplaceBulk for T
    where
        T: ~const IntoBulk<IntoBulk = B>,
        B: ~const CollectNearest<Nearest = N>,
        N: ~const IntoBulk<IntoBulk: InplaceBulk<Item = T::Item>>
    {
        type _IntoInplaceBulk = N::IntoBulk;

        fn _into_inplace_bulk(self) -> Self::_IntoInplaceBulk
        {
            self.into_bulk()
                .collect_nearest()
                .into_bulk()
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3, 4, 5];

        let mut b = a.into_bulk()
            .map(|x| 6 - x)
            .into_inplace_bulk();

        b.swap_inplace(1, 3);

        let c = b.collect_nearest();

        assert_eq!(c, [5, 2, 3, 4, 1])
    }
}