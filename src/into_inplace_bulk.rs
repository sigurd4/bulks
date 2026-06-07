use crate::{InplaceBulk, IntoBulk};

pub trait IntoInplaceBulk: IntoBulk
{
    type IntoInplaceBulk: InplaceBulk;

    fn into_inplace_bulk(self) -> Self::IntoInplaceBulk;
}
#[cfg(feature = "alloc")]
impl<I> IntoInplaceBulk for I
where
    I: IntoBulk
{
    type IntoInplaceBulk = <I as private::_IntoInplace>::_IntoInplace;

    fn into_inplace_bulk(self) -> Self::IntoInplaceBulk
    {
        self._into_inplace()
    }
}

#[cfg(not(feature = "alloc"))]
impl<I> IntoInplaceBulk for I
where
    I: IntoBulk<IntoBulk: InplaceBulk>
{
    type IntoInplaceBulk = <I as private::_IntoInplace>::_IntoInplace;

    fn into_inplace_bulk(self) -> Self::IntoInplaceBulk
    {
        self._into_inplace()
    }
}

mod private
{
    use array_trait::same::Same;

    use crate::{CollectNearest, InplaceBulk, IntoBulk};

    pub trait _IntoInplace
    {
        type _IntoInplace: InplaceBulk;

        fn _into_inplace(self) -> Self::_IntoInplace;
    }
    #[cfg(feature = "alloc")]
    impl<I> _IntoInplace for I
    where
        I: IntoBulk
    {
        default type _IntoInplace = <I::IntoBulk as CollectNearest>::Nearest;

        default fn _into_inplace(self) -> Self::_IntoInplace
        {
            self.into_bulk()
                .collect_nearest()
                .into_bulk()
        }
    }
    impl<I> _IntoInplace for I
    where
        I: IntoBulk<IntoBulk: InplaceBulk>
    {
        type _IntoInplace = I::IntoBulk;

        fn _into_inplace(self) -> Self::_IntoInplace
        {
            self.into_bulk()
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