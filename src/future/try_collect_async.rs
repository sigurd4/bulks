use core::{
    marker::PhantomData,
    ops::{FromResidual, Residual, Try},
    pin::Pin,
    task::{Context, Poll}
};

use array_trait::AsSlice;

use crate::{
    AsBulkMut, Bulk, BulkLength, CollectionAdapter, FromBulk, TryCollectionStrategy,
    util::{Buffer, BufferableBulk, MaybeDone}
};

pub struct TryCollectAsync<B, C, A>
where
    B: BufferableBulk,
    B::Item: Future<Output: Try<Residual: Residual<C>>>,
    C: FromBulk<A>,
    A: CollectionAdapter<Elem = <<B::Item as Future>::Output as Try>::Output> + TryCollectionStrategy<B::MinLength, B::MaxLength, C> + ?Sized
{
    queue: B::IntoIter,
    tasks: Buffer<MaybeDone<B::Item>, BulkLength<B>>,
    marker: PhantomData<(C, A)>
}
impl<B, C, A> TryCollectAsync<B, C, A>
where
    B: BufferableBulk,
    B::Item: Future<Output: Try<Residual: Residual<C>>>,
    C: FromBulk<A>,
    A: CollectionAdapter<Elem = <<B::Item as Future>::Output as Try>::Output> + TryCollectionStrategy<B::MinLength, B::MaxLength, C> + ?Sized
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

    fn cancel(self: Pin<&mut Self>)
    {
        for task in self.tasks().bulk_pin_mut()
        {
            task.cancel()
        }
    }

    fn collect(self: Pin<&mut Self>) -> <<<B::Item as Future>::Output as Try>::Residual as Residual<C>>::TryType
    {
        unsafe { self.map_unchecked_mut(|this| &mut this.tasks) }
            .full_bulk_pin_mut()
            .map(|x| x.take_output().unwrap())
            .try_collect()
    }
}
impl<B, C, A> Future for TryCollectAsync<B, C, A>
where
    B: BufferableBulk,
    B::Item: Future<Output: Try<Residual: Residual<C>>>,
    C: FromBulk<A>,
    A: CollectionAdapter<Elem = <<B::Item as Future>::Output as Try>::Output> + TryCollectionStrategy<B::MinLength, B::MaxLength, C> + ?Sized
{
    type Output = <<<B::Item as Future>::Output as Try>::Residual as Residual<C>>::TryType;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let mut ready = match self.as_mut().tasks().bulk_pin_mut().try_fold(true, |ready, mut task| {
            if !task.is_taken()
            {
                if !task.as_mut().poll(cx).is_ready()
                {
                    return Ok(false);
                }

                if let Some(residual) = task.take_residual()
                {
                    return Err(residual);
                }
            }

            Ok(ready)
        })
        {
            Err(residual) =>
            {
                self.cancel();
                return Poll::Ready(FromResidual::from_residual(residual));
            }
            Ok(ready) => ready
        };

        while let Some(value) = self.as_mut().queue().next()
        {
            let mut task = self.as_mut().push_task(value);
            if !task.as_mut().poll(cx).is_ready()
            {
                ready = false
            }
            else if let Some(residual) = task.take_residual()
            {
                self.cancel();
                return Poll::Ready(FromResidual::from_residual(residual));
            }
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
            let b: Result<[_; 4], _> = a.into_bulk().map(async |x| x.parse::<u32>()).try_collect_async().await;
            assert_eq!(b, Ok([1, 2, 3, 4]));
        })
    }
}
