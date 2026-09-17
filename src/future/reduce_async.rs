use core::{
    pin::Pin,
    task::{Context, Poll}
};

use array_trait::AsSlice;

use crate::{
    AsBulkMut, Bulk, BulkLength,
    util::{Buffer, BufferableBulk, MaybeDone}
};

pub struct ReduceAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item, B::Item), Output: Future<Output = B::Item>>
{
    queue: B::IntoIter,
    tasks: Buffer<MaybeDone<F::Output>, BulkLength<B>>,
    action: F
}
impl<B, F> ReduceAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item, B::Item), Output: Future<Output = B::Item>>
{
    pub(crate) fn new(bulk: B, action: F) -> Self
    where
        B: IntoIterator
    {
        let len = bulk.length();
        Self {
            queue: bulk.into_iter(),
            tasks: Buffer::new(len),
            action
        }
    }

    fn tasks(self: Pin<&mut Self>) -> Pin<&mut [MaybeDone<F::Output>]>
    {
        unsafe { self.map_unchecked_mut(|this| this.tasks.as_mut_slice()) }
    }

    fn task(self: Pin<&mut Self>, i: usize) -> Pin<&mut MaybeDone<F::Output>>
    {
        unsafe { self.map_unchecked_mut(|this| &mut this.tasks.as_mut_slice()[i]) }
    }

    fn action(self: Pin<&mut Self>) -> &mut F
    {
        unsafe { &mut self.get_unchecked_mut().action }
    }

    fn queue(self: Pin<&mut Self>) -> &mut B::IntoIter
    {
        unsafe { &mut self.get_unchecked_mut().queue }
    }

    fn push_task(self: Pin<&mut Self>, lhs: B::Item, rhs: Option<B::Item>) -> Pin<&mut MaybeDone<F::Output>>
    {
        match rhs
        {
            Some(rhs) =>
            unsafe { self.map_unchecked_mut(|this| this.tasks.push_mut(MaybeDone::Future((this.action)(lhs, rhs)))) },
            None =>
            unsafe { self.map_unchecked_mut(|this| this.tasks.push_mut(MaybeDone::Done(lhs))) }
        }
    }
}

impl<B, F> Future for ReduceAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item, B::Item), Output: Future<Output = B::Item>>
{
    type Output = Option<B::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let mut ready = true;
        while let Some(lhs) = self.as_mut().queue().next()
        {
            let rhs = self.as_mut().queue().next();
            let task = self.as_mut().push_task(lhs, rhs);
            ready &= task.poll(cx).is_ready()
        }

        let len = self.tasks.len();

        let mut i = 0;
        let mut j = None;
        while i < len
        {
            if !self.as_mut().task(i).is_taken()
            {
                if self.as_mut().task(i).poll(cx).is_ready()
                {
                    match j.take()
                    {
                        None => j = Some(i),
                        Some(j) =>
                        {
                            let lhs = self.as_mut().task(j).take_output().unwrap();
                            let rhs = self.as_mut().task(i).take_output().unwrap();
                            let future = self.as_mut().action()(lhs, rhs);
                            self.as_mut().task(i).restart(future);
                            continue;
                        }
                    }
                }
                else
                {
                    ready = false
                }
            }
            i += 1
        }

        if ready
        {
            match j
            {
                Some(j) => Poll::Ready(self.task(j).take_output()),
                None => Poll::Ready(None)
            }
        }
        else
        {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod test
{
    use core::time::Duration;

    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3];

        let n = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { a.into_bulk().reduce_async(async |a, b| a + b).await });

        assert_eq!(n, Some(1 + 2 + 3))
    }
}
