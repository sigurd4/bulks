use core::{marker::Destruct, ptr::Thin};

use array_trait::length::LengthValue;

use crate::{Bulk, IntoBulk, StaticBulk};

pub const trait RandomAccessBulk: ~const Bulk
{
    type ItemPointee: Sized + Thin;
    type EachRef<'a>: ~const RandomAccessBulk<ItemPointee = Self::ItemPointee, Item = &'a Self::ItemPointee, EachRef<'a> = Self::EachRef<'a>> + 'a + ~const Destruct
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(bulk: &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a;
}

pub const trait InplaceBulk: ~const RandomAccessBulk
{
    type EachMut<'a>: ~const InplaceBulk<ItemPointee = Self::ItemPointee, Item = &'a mut Self::ItemPointee, EachRef<'a> = Self::EachRef<'a>, EachMut<'a> = Self::EachMut<'a>> + 'a + ~const Destruct
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_mut<'a>(bulk: &'a mut Self) -> Self::EachMut<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a;
}

pub(crate) const trait RandomAccessBulkSpec: Bulk
{
    fn _get<'a, L>(bulk: &'a Self, i: L) -> Option<&'a <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: ~const RandomAccessBulk + 'a;

    fn _get_many<'a, NN, const N: usize>(bulk: &'a Self, i: NN) -> [Option<&'a <Self as RandomAccessBulk>::ItemPointee>; N]
    where
        Self: ~const RandomAccessBulk + 'a,
        NN: ~const IntoBulk<Item = usize, IntoBulk: ~const Bulk + StaticBulk<Array<()> = [(); N]>>;
}
impl<I> const RandomAccessBulkSpec for I
where
    I: Bulk + ?Sized
{
    default fn _get<'a, L>(bulk: &'a Self, i: L) -> Option<&'a <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: ~const RandomAccessBulk + 'a
    {
        bulk.each_ref().nth(i)
    }

    default fn _get_many<'a, NN, const N: usize>(bulk: &'a Self, i: NN) -> [Option<&'a <Self as RandomAccessBulk>::ItemPointee>; N]
    where
        Self: ~const RandomAccessBulk + 'a,
        NN: ~const IntoBulk<Item = usize, IntoBulk: ~const Bulk + StaticBulk<Array<()> = [(); N]>>
    {
        bulk.each_ref().many(i)
    }
}

pub(crate) const trait InplaceBulkSpec: Bulk
{
    fn _get_mut<'a, L>(bulk: &'a mut Self, i: L) -> Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: ~const InplaceBulk + 'a;

    fn _get_many_mut<'a, NN, const N: usize>(bulk: &'a mut Self, n: NN) -> [Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>; N]
    where
        Self: ~const InplaceBulk + 'a,
        NN: ~const IntoBulk<Item = usize, IntoBulk: ~const Bulk + StaticBulk<Array<()> = [(); N]>>;
}
impl<I> const InplaceBulkSpec for I
where
    I: Bulk + ?Sized
{
    default fn _get_mut<'a, L>(bulk: &'a mut Self, i: L) -> Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: ~const InplaceBulk + 'a
    {
        bulk.each_mut().nth(i)
    }

    default fn _get_many_mut<'a, NN, const N: usize>(bulk: &'a mut Self, i: NN) -> [Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>; N]
    where
        Self: ~const InplaceBulk + 'a,
        NN: ~const IntoBulk<Item = usize, IntoBulk: ~const Bulk + StaticBulk<Array<()> = [(); N]>>
    {
        bulk.each_mut().many(i)
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn test_swap()
    {
        fn swaps<B>(bulk: &mut B)
        where
            B: InplaceBulk
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

    #[test]
    fn test_many()
    {
        let a = [1, 2, 3, 4];
        let mut bulk = a.into_bulk();
        let [x1, x2, x3] = bulk.get_many_mut([1, 0, 3]).map(Option::unwrap);

        std::mem::swap(x1, x2);
        std::mem::swap(x2, x3);

        let b = bulk.collect_nearest();

        println!("{b:?}");
    }
}