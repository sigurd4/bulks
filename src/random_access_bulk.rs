use core::marker::Destruct;

use array_trait::length::LengthValue;

use crate::Bulk;

pub const trait RandomAccessBulk<'a>: ~const Bulk + 'a
{
    type ItemRef: 'a + Copy + ~const Destruct;
    type EachRef: ~const RandomAccessBulk<'a, Item = Self::ItemRef, ItemRef = Self::ItemRef, EachRef = Self::EachRef> + 'a;

    fn each_ref(bulk: &'a Self) -> Self::EachRef;
}

pub const trait InplaceBulk<'a>: ~const RandomAccessBulk<'a>
{
    type ItemMut: 'a + ~const Destruct;
    type EachMut: ~const InplaceBulk<'a, Item = Self::ItemMut, ItemMut = Self::ItemMut, ItemRef = Self::ItemRef, EachMut = Self::EachMut, EachRef = Self::EachRef> + 'a;

    fn each_mut(bulk: &'a mut Self) -> Self::EachMut;
}

pub(crate) const trait RandomAccessBulkSpec<'a>: Bulk
{
    fn _get<L>(bulk: &'a Self, i: L) -> Option<Self::ItemRef>
    where
        L: LengthValue,
        Self: ~const RandomAccessBulk<'a> + 'a;
}
impl<'a, I> const RandomAccessBulkSpec<'a> for I
where
    I: Bulk + ?Sized
{
    default fn _get<L>(bulk: &'a Self, i: L) -> Option<<Self as RandomAccessBulk<'a>>::ItemRef>
    where
        L: LengthValue,
        Self: ~const RandomAccessBulk<'a> + 'a
    {
        bulk.each_ref().nth(i)
    }
}

pub(crate) const trait InplaceBulkSpec<'a>: Bulk
{
    fn _get_mut<L>(bulk: &'a mut Self, i: L) -> Option<Self::ItemMut>
    where
        L: LengthValue,
        Self: ~const InplaceBulk<'a> + 'a;
}
impl<'a, I> const InplaceBulkSpec<'a> for I
where
    I: Bulk + ?Sized
{
    default fn _get_mut<L>(bulk: &'a mut Self, i: L) -> Option<<Self as InplaceBulk<'a>>::ItemMut>
    where
        L: LengthValue,
        Self: ~const InplaceBulk<'a> + 'a
    {
        bulk.each_mut().nth(i)
    }
}

#[cfg(test)]
mod test
{
    use core::borrow::BorrowMut;

    use crate::*;

    #[test]
    fn it_works()
    {
        fn swaps<B>(bulk: &mut B)
        where
            B: for<'a> InplaceBulk<'a, ItemMut: BorrowMut<B::Item>>
        {
            bulk.swap_inplace(0, 3);
            bulk.swap_inplace(1, 2);
        }

        let a = [1, 2, 3, 4];
        let mut bulk = a.into_bulk();
        swaps(&mut bulk);
        let b = bulk.collect_nearest();
        println!("{b:?}")
    }
}