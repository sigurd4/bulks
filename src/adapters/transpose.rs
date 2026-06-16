use core::ops::Try;

use crate::{Bulk, IntoBulk};

pub const trait TransposableBulk: ~const private::TransposableBulk
{

}
impl<I> const TransposableBulk for I
where
    I: ~const private::TransposableBulk
{

}

pub struct Transpose<I>
where
    I: TransposableBulk
{
    bulks: I::Rows
}

impl<I> Transpose<I>
where
    I: TransposableBulk
{
    pub(crate) const fn new(bulk: I) -> Self
    where
        I: ~const TransposableBulk
    {
        Self {
            bulks: I::rows(bulk)
        }
    }
}

impl<I, T> IntoIterator for Transpose<I>
where
    I: TransposableBulk<Item: IntoBulk<Item = T>>
{
    type IntoIter = I::RowsIntoIter;
    type Item = <I::RowsIntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulks } = self;
        
        I::rows_into_iter(bulks)
    }
}

impl<I, T> Bulk for Transpose<I>
where
    I: private::TransposableBulk<Item: IntoBulk<Item = T>>
{
    type MinLength = <<I::Item as IntoBulk>::IntoBulk as Bulk>::MinLength;
    type MaxLength = <<I::Item as IntoBulk>::IntoBulk as Bulk>::MaxLength;

    fn len(&self) -> usize
    {
        self.bulks.each_ref()
            .map(|bulk| bulk.len())
            .reduce(Ord::min)
            .unwrap_or(0)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.into_iter().for_each(f)
    }

    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().try_for_each(f)
    }
}

mod private
{
    use crate::{Bulk, CollectNearest, InplaceBulk, IntoBulk, Map};

    pub trait TransposableIntoIter: Bulk<Item: IntoBulk>
    {
        type RowsIntoIter: ExactSizeIterator<Item: IntoBulk<Item = <Self::Item as IntoIterator>::Item>>;

        fn rows_into_iter(bulks: Self::Rows) -> Self::RowsIntoIter
        where
            Self: TransposableBulk;
    }
    pub const trait TransposableBulk: ~const Bulk<Item: IntoBulk> + TransposableIntoIter
    {
        type Rows: InplaceBulk<Item = <Self::Item as IntoBulk>::IntoBulk, ItemPointee = <Self::Item as IntoBulk>::IntoBulk>;

        fn rows(bulks: Self) -> Self::Rows;
    }

    impl<I, T, N> TransposableIntoIter for I
    where
        I: Bulk<Item: IntoBulk<Item = T>>,
        Map<I, fn(I::Item) -> <I::Item as IntoBulk>::IntoBulk>: CollectNearest<Nearest = N>,
        N: IntoBulk<IntoBulk: InplaceBulk<Item = <Self::Item as IntoBulk>::IntoBulk, ItemPointee = <Self::Item as IntoBulk>::IntoBulk>>,
        IntoIter<N::IntoBulk>: ExactSizeIterator<Item: IntoBulk<Item = <Self::Item as IntoIterator>::Item>>,
        <<Map<N::IntoBulk, fn(<I::Item as IntoBulk>::IntoBulk) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest as IntoIterator>::IntoIter: ExactSizeIterator
    {
        type RowsIntoIter = IntoIter<N::IntoBulk>;

        fn rows_into_iter(bulks: <Self as TransposableBulk>::Rows) -> Self::RowsIntoIter
        {
            IntoIter::<N::IntoBulk> {
                iters: bulks.map(IntoIterator::into_iter as fn(_) -> _)
                    .collect_nearest()
                    .into_bulk()
            }
        }
    }
    impl<I, T, N> const TransposableBulk for I
    where
        I: ~const Bulk<Item: IntoBulk<Item = T>>,
        Map<I, fn(I::Item) -> <I::Item as IntoBulk>::IntoBulk>: ~const CollectNearest<Nearest = N>,
        N: ~const IntoBulk<IntoBulk: InplaceBulk<Item = <Self::Item as IntoBulk>::IntoBulk, ItemPointee = <Self::Item as IntoBulk>::IntoBulk>>,
        IntoIter<N::IntoBulk>: ExactSizeIterator<Item: IntoBulk<Item = <Self::Item as IntoIterator>::Item>>,
        <<Map<N::IntoBulk, fn(<I::Item as IntoBulk>::IntoBulk) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest as IntoIterator>::IntoIter: ExactSizeIterator
    {
        type Rows = <<Map<I, fn(I::Item) -> <I::Item as IntoBulk>::IntoBulk> as CollectNearest>::Nearest as IntoBulk>::IntoBulk;

        fn rows(bulks: Self) -> Self::Rows
        {
            bulks.map(IntoBulk::into_bulk as fn(_) -> _)
                .collect_nearest()
                .into_bulk()
        }
    }

    pub struct IntoIter<I>
    where
        I: Bulk<Item: IntoBulk>,
        Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter>: CollectNearest<Nearest: IntoBulk>
    {
        pub(super) iters: <<Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest as IntoBulk>::IntoBulk
    }
    impl<I, R> Iterator for IntoIter<I>
    where
        I: Bulk<Item: IntoBulk>,
        <<I as IntoIterator>::Item as IntoIterator>::IntoIter: 'static,
        Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter>: CollectNearest<Nearest: IntoBulk<IntoBulk: InplaceBulk<ItemPointee = <I::Item as IntoIterator>::IntoIter>> + 'static>,
        Map<<<<Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest as IntoBulk>::IntoBulk as InplaceBulk>::EachMut<'static>, fn(&'static mut <I::Item as IntoIterator>::IntoIter) -> Option<<I::Item as IntoIterator>::Item>>: CollectNearest<Item = Option<<I::Item as IntoIterator>::Item>, TryNearest = R>,
    {
        type Item = R;

        fn next(&mut self) -> Option<Self::Item>
        {
            let iters: &mut <<Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest as IntoBulk>::IntoBulk  = unsafe {
                core::mem::transmute(&mut self.iters)
            };
            iters.each_mut()
                .map(Iterator::next as fn(&'static mut _) -> Option<_>)
                .try_collect_nearest()
        }
    }
    impl<I, R> ExactSizeIterator for IntoIter<I>
    where
        I: Bulk<Item: IntoBulk>,
        <<I as IntoIterator>::Item as IntoIterator>::IntoIter: 'static,
        Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter>: CollectNearest<Nearest: IntoBulk<IntoBulk: InplaceBulk<ItemPointee = <I::Item as IntoIterator>::IntoIter>> + 'static>,
        Map<<<<Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest as IntoBulk>::IntoBulk as InplaceBulk>::EachMut<'static>, fn(&'static mut <I::Item as IntoIterator>::IntoIter) -> Option<<I::Item as IntoIterator>::Item>>: CollectNearest<Item = Option<<I::Item as IntoIterator>::Item>, TryNearest = R>,
    {
        fn len(&self) -> usize
        {
            self.iters.each_ref()
                .map(|iter| iter.len())
                .reduce(Ord::min)
                .unwrap_or(0)
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [[1, 4], [2, 5], [3, 6]];
        let b = a.into_bulk()
            .transpose()
            .collect_array();

        println!("{b:?}")
    }
}