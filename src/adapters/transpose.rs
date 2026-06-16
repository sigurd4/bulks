use crate::{Bulk, InplaceBulk, CollectNearest, IntoBulk};

pub struct Transpose<I>
where
    I: Bulk<Item: IntoBulk>
{
    bulk: I
}

mod private
{
    use core::{convert::Infallible, ops::Residual};

use array_trait::length;

use crate::{Bulk, CollectNearest, InplaceBulk, IntoBulk, Map};

    pub struct IntoIter<I>
    where
        I: Bulk<Item: IntoBulk>,
        Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter>: CollectNearest
    {
        iters: <Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest
    }
    impl<I> IntoIter<I>
    where
        I: Bulk<Item: IntoBulk>,
        Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter>: CollectNearest
    {
        pub fn new(bulk: I) -> Self
        {
            Self {
                iters: bulk.map(IntoIterator::into_iter as fn(_) -> _)
                    .collect_nearest()
            }
        }
    }
    impl<I, R> Iterator for IntoIter<I>
    where
        I: Bulk<Item: IntoBulk>,
        <I::Item as IntoIterator>::IntoIter: 'static,
        Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter>: CollectNearest<Nearest: InplaceBulk<ItemPointee = <I::Item as IntoIterator>::IntoIter> + 'static>,
        Map<<<Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest as InplaceBulk>::EachMut<'static>, fn(&'static mut <I::Item as IntoIterator>::IntoIter) -> Option<<I::Item as IntoIterator>::Item>>: CollectNearest<Item = Option<<I::Item as IntoIterator>::Item>, TryNearest = R>,
    {
        type Item = R;

        fn next(&mut self) -> Option<Self::Item>
        {
            let iters: &mut <Map<I, fn(I::Item) -> <I::Item as IntoIterator>::IntoIter> as CollectNearest>::Nearest  = unsafe {
                core::mem::transmute(&mut self.iters)
            };
            iters.each_mut()
                .map(Iterator::next as fn(&'static mut _) -> Option<_>)
                .try_collect_nearest()
        }
    }
}

impl<I, T> IntoIterator for Transpose<I>
where
    I: Bulk<Item: IntoBulk<Item = T>>,
    private::IntoIter<I>: Iterator<Item = <I::Item as IntoIterator>::Item>
{
    type IntoIter = private::IntoIter<I>;
    type Item = <I::Item as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk } = self;
        
        private::IntoIter::new(bulk)
    }
}