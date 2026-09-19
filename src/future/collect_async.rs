use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll}
};

use array_trait::AsSlice;

use crate::{
    AsBulkMut, Bulk, BulkLength, CollectionAdapter, CollectionStrategy, FromBulk,
    util::{Buffer, BufferableBulk, MaybeDone}
};

pub struct CollectAsync<B, C, A>
where
    B: BufferableBulk,
    B::Item: Future,
    C: FromBulk<A>,
    A: CollectionAdapter<Elem = <B::Item as Future>::Output> + CollectionStrategy<B::MinLength, B::MaxLength, C> + ?Sized
{
    queue: B::IntoIter,
    tasks: Buffer<MaybeDone<B::Item>, BulkLength<B>>,
    marker: PhantomData<(C, A)>
}
impl<B, C, A> CollectAsync<B, C, A>
where
    B: BufferableBulk,
    B::Item: Future,
    C: FromBulk<A>,
    A: CollectionAdapter<Elem = <B::Item as Future>::Output> + CollectionStrategy<B::MinLength, B::MaxLength, C> + ?Sized
{
    pub(crate) fn new(bulk: B) -> Self
    {
        let len = bulk.length();
        Self {
            queue: bulk.into_iter(),
            tasks: Buffer::new(len),
            marker: PhantomData
        }
    }

    fn tasks(self: Pin<&mut Self>) -> Pin<&mut [MaybeDone<B::Item>]>
    {
        unsafe { self.map_unchecked_mut(|this| this.tasks.as_mut_slice()) }
    }

    fn queue(self: Pin<&mut Self>) -> &mut B::IntoIter
    {
        unsafe { &mut self.get_unchecked_mut().queue }
    }

    fn push_task(self: Pin<&mut Self>, value: B::Item) -> Pin<&mut MaybeDone<B::Item>>
    {
        unsafe { self.map_unchecked_mut(|this| this.tasks.push_mut(MaybeDone::Future(value))) }
    }

    fn collect(self: Pin<&mut Self>) -> C
    {
        unsafe { self.map_unchecked_mut(|this| &mut this.tasks) }
            .full_bulk_pin_mut()
            .map(|x| x.take_output().unwrap())
            .collect()
    }
}
impl<B, C, A> Future for CollectAsync<B, C, A>
where
    B: BufferableBulk,
    B::Item: Future,
    C: FromBulk<A>,
    A: CollectionAdapter<Elem = <B::Item as Future>::Output> + CollectionStrategy<B::MinLength, B::MaxLength, C> + ?Sized
{
    type Output = C;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let mut ready = self.as_mut().tasks().bulk_pin_mut().fold(true, |ready, task| ready & task.poll(cx).is_ready());

        while let Some(value) = self.as_mut().queue().next()
        {
            let task = self.as_mut().push_task(value);
            ready &= task.poll(cx).is_ready()
        }

        if ready { Poll::Ready(self.collect()) } else { Poll::Pending }
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = ["1", "2", "3", "4"];

        tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
            let b: [_; 4] = a.into_bulk().map(async |x| x.parse::<u32>().unwrap()).collect_async().await;
            assert_eq!(b, [1, 2, 3, 4]);
        })
    }
}
