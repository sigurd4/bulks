use crate::{InplaceBulk, IntoBulk};

pub const trait IntoInplaceBulk
{
    type IntoInplaceBulk: InplaceBulk;

    fn into_inplace_bulk(self) -> Self::IntoInplaceBulk;
}
impl<I> const IntoInplaceBulk for I
where
    I: ~const private::_IntoInplace<_IntoInplace: ~const IntoBulk<IntoBulk: InplaceBulk>>
{
    type IntoInplaceBulk = <<I as private::_IntoInplace>::_IntoInplace as IntoBulk>::IntoBulk;

    fn into_inplace_bulk(self) -> Self::IntoInplaceBulk
    {
        self._into_inplace()
            .into_bulk()
    }
}

mod private
{
    use array_trait::same::Same;

    use crate::{CollectNearest, InplaceBulk, IntoBulk};

    pub const trait _IntoInplace
    {
        type _IntoInplace;

        fn _into_inplace(self) -> Self::_IntoInplace;
    }
    impl<I> const _IntoInplace for I
    {
        default type _IntoInplace = I;

        default fn _into_inplace(self) -> Self::_IntoInplace
        {
            unsafe {
                self.same()
                    .unwrap_unchecked()
            }
        }
    }
    impl<I, B, N> const _IntoInplace for I
    where
        I: ~const IntoBulk<IntoBulk = B>,
        B: ~const CollectNearest<Nearest = N>
    {
        type _IntoInplace = N;

        fn _into_inplace(self) -> Self::_IntoInplace
        {
            self.into_bulk()
                .collect_nearest()
        }
    }

    pub const trait __IntoInplace
    {
        type __IntoInplace;

        fn __into_inplace(self) -> Self::__IntoInplace;
    }
    impl<I> const __IntoInplace for I
    {
        default type __IntoInplace = <I as _IntoInplace>::_IntoInplace;

        default fn __into_inplace(self) -> Self::__IntoInplace
        {
            unsafe {
                self._into_inplace()
                    .same()
                    .unwrap_unchecked()
            }
        }
    }
    impl<I, B> const __IntoInplace for I
    where
        I: IntoBulk<IntoBulk = B>,
        B: InplaceBulk
    {
        type __IntoInplace = I;

        fn __into_inplace(self) -> Self::__IntoInplace
        {
            self
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