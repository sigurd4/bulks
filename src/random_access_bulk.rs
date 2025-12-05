use core::marker::Destruct;

use array_trait::length::LengthValue;

use crate::Bulk;

pub const trait RandomAccessBulk<'a>: ~const Bulk + 'a
{
    type ItemRef: 'a + Copy + ~const Destruct;
    type EachRef: ~const RandomAccessBulk<'a, Item = Self::ItemRef, ItemRef = Self::ItemRef, EachRef = Self::EachRef> + 'a;

    fn each_ref(bulk: &'a Self) -> Self::EachRef;
}

pub const trait RandomAccessBulkMut<'a>: ~const RandomAccessBulk<'a>
{
    type ItemMut: 'a + ~const Destruct;
    type EachMut: ~const RandomAccessBulkMut<'a, Item = Self::ItemMut, ItemMut = Self::ItemMut, ItemRef = Self::ItemRef, EachMut = Self::EachMut, EachRef = Self::EachRef> + 'a;

    fn each_mut(bulk: &'a mut Self) -> Self::EachMut;
}

pub(crate) const trait RandomAccessBulkSpec<'a>: ~const RandomAccessBulk<'a>
{
    fn _get<L>(bulk: &'a Self, i: L) -> Option<Self::ItemRef>
    where
        L: LengthValue;
}
impl<'a, I> const RandomAccessBulkSpec<'a> for I
where
    I: ~const RandomAccessBulk<'a> + ?Sized
{
    default fn _get<L>(bulk: &'a Self, i: L) -> Option<Self::ItemRef>
    where
        L: LengthValue
    {
        bulk.each_ref().nth(i)
    }
}

pub(crate) const trait RandomAccessBulkMutSpec<'a>: ~const RandomAccessBulkMut<'a>
{
    fn _get_mut<L>(bulk: &'a mut Self, i: L) -> Option<Self::ItemMut>
    where
        L: LengthValue;
}
impl<'a, I> const RandomAccessBulkMutSpec<'a> for I
where
    I: ~const RandomAccessBulkMut<'a> + ?Sized
{
    default fn _get_mut<L>(bulk: &'a mut Self, i: L) -> Option<Self::ItemMut>
    where
        L: LengthValue
    {
        bulk.each_mut().nth(i)
    }
}