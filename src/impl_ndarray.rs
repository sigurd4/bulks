use core::ops::Try;

use ::ndarray::{ArrayBase, ArrayRef, Data, DataMut, Dimension, RawData};

use crate::{Bulk, DoubleEndedBulk, IntoBulk};

pub mod ndarray
{
    use ndarray::{ArrayBase, ArrayRef, Dimension, RawData};

    pub struct IntoBulk<S, D, A = <S as RawData>::Elem>
    where
        S: RawData<Elem = A>,
        D: Dimension,
        ArrayBase<S, D, A>: IntoIterator<IntoIter: ExactSizeIterator>
    {
        pub(super) ndarray: ArrayBase<S, D, A>
    }

    pub struct Bulk<'a, A, D>
    where
        D: Dimension
    {
        pub(super) ndarray: &'a ArrayRef<A, D>
    }
    pub struct BulkMut<'a, A, D>
    where
        D: Dimension
    {
        pub(super) ndarray: &'a mut ArrayRef<A, D>
    }
}

impl<S, D, A> IntoBulk for ArrayBase<S, D, A>
where
    S: RawData<Elem = A>,
    D: Dimension,
    ArrayBase<S, D, A>: IntoIterator<IntoIter: ExactSizeIterator>
{
    type IntoBulk = ndarray::IntoBulk<S, D, A>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        ndarray::IntoBulk {
            ndarray: self
        }
    }
}
impl<'a, S, D, A> IntoBulk for &'a ArrayBase<S, D, A>
where
    S: Data<Elem = A>,
    D: Dimension
{
    type IntoBulk = ndarray::Bulk<'a, A, D>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        ndarray::Bulk {
            ndarray: self
        }
    }
}
impl<'a, S, D, A> IntoBulk for &'a mut ArrayBase<S, D, A>
where
    S: DataMut<Elem = A>,
    D: Dimension
{
    type IntoBulk = ndarray::BulkMut<'a, A, D>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        ndarray::BulkMut {
            ndarray: self
        }
    }
}
impl<'a, A, D> IntoBulk for &'a ArrayRef<A, D>
where
    D: Dimension
{
    type IntoBulk = ndarray::Bulk<'a, A, D>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        ndarray::Bulk {
            ndarray: self
        }
    }
}
impl<'a, A, D> IntoBulk for &'a mut ArrayRef<A, D>
where
    D: Dimension
{
    type IntoBulk = ndarray::BulkMut<'a, A, D>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        ndarray::BulkMut {
            ndarray: self
        }
    }
}

impl<S, D, A> IntoIterator for ndarray::IntoBulk<S, D, A>
where
    S: RawData<Elem = A>,
    D: Dimension,
    ArrayBase<S, D, A>: IntoIterator<IntoIter: ExactSizeIterator>
{
    type Item = <ArrayBase<S, D, A> as IntoIterator>::Item;
    type IntoIter = <ArrayBase<S, D, A> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        self.ndarray.into_iter()
    }
}
impl<'a, A, D> IntoIterator for ndarray::Bulk<'a, A, D>
where
    D: Dimension
{
    type Item = &'a A;
    type IntoIter = ::ndarray::iter::Iter<'a, A, D>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.ndarray.iter()
    }
}
impl<'a, A, D> IntoIterator for ndarray::BulkMut<'a, A, D>
where
    D: Dimension
{
    type Item = &'a mut A;
    type IntoIter = ::ndarray::iter::IterMut<'a, A, D>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.ndarray.iter_mut()
    }
}

impl<S, D, A> Bulk for ndarray::IntoBulk<S, D, A>
where
    S: RawData<Elem = A>,
    D: Dimension,
    ArrayBase<S, D, A>: IntoIterator<IntoIter: ExactSizeIterator>
{
    fn len(&self) -> usize
    {
        self.ndarray.len()
    }

    fn for_each<F>(self, f: F)
    where 
        F: FnMut(Self::Item)
    {
        self.into_iter().for_each(f)
    }

    fn try_for_each<F, R>(self, f: F) -> R
    where
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().try_for_each(f)
    }
}
impl<'a, A, D> Bulk for ndarray::Bulk<'a, A, D>
where
    D: Dimension
{
    fn len(&self) -> usize
    {
        self.ndarray.len()
    }

    fn for_each<F>(self, f: F)
    where 
        F: FnMut(Self::Item)
    {
        self.into_iter().for_each(f)
    }

    fn try_for_each<F, R>(self, f: F) -> R
    where
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().try_for_each(f)
    }
}
impl<'a, A, D> Bulk for ndarray::BulkMut<'a, A, D>
where
    D: Dimension
{
    fn len(&self) -> usize
    {
        self.ndarray.len()
    }

    fn for_each<F>(self, f: F)
    where 
        F: FnMut(Self::Item)
    {
        self.into_iter().for_each(f)
    }

    fn try_for_each<F, R>(self, f: F) -> R
    where
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().try_for_each(f)
    }
}

impl<S, D, A> DoubleEndedBulk for ndarray::IntoBulk<S, D, A>
where
    S: RawData<Elem = A>,
    D: Dimension,
    ArrayBase<S, D, A>: IntoIterator<IntoIter: ExactSizeIterator + DoubleEndedIterator>
{
    fn rev_for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item)
    {
        self.into_iter().rev().for_each(f)
    }

    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().rev().try_for_each(f)
    }
}
impl<'a, A, D> DoubleEndedBulk for ndarray::Bulk<'a, A, D>
where
    D: Dimension,
    ::ndarray::iter::Iter<'a, A, D>: DoubleEndedIterator<Item = &'a A>
{
    fn rev_for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item)
    {
        self.into_iter().rev().for_each(f)
    }

    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().rev().try_for_each(f)
    }
}
impl<'a, A, D> DoubleEndedBulk for ndarray::BulkMut<'a, A, D>
where
    D: Dimension,
    ::ndarray::iter::IterMut<'a, A, D>: DoubleEndedIterator<Item = &'a mut A>
{
    fn rev_for_each<F>(self, f: F)
    where
        F: FnMut(Self::Item)
    {
        self.into_iter().rev().for_each(f)
    }

    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().rev().try_for_each(f)
    }
}

#[cfg(test)]
mod test
{
    use crate::{AsBulk, Bulk};

    #[test]
    fn it_works()
    {
        let mut a = ndarray::arr2(&[
            [1, 2, 3],
            [4, 5, 6]
        ]);

        for mut row in a.rows_mut()
        {
            row.bulk_mut()
                .rev()
                .swap::<u8>(0, 2);
        }

        println!("{a:?}")
    }
}