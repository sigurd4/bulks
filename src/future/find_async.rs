use core::{
    pin::Pin,
    task::{Context, Poll}
};

use array_trait::AsSlice;

use crate::{
    AsBulkMut, Bulk, BulkLength,
    util::{Buffer, BufferableBulk, MaybeDone}
};

pub struct FindAsync<B, F, Y>
where
    B: BufferableBulk,
    F: FnMut(&B::Item) -> Y,
    Y: Future<Output = bool>
{
    queue: B::IntoIter,
    tasks: Buffer<Task<B::Item, Y>, BulkLength<B>>,
    action: F
}
impl<B, F, Y> FindAsync<B, F, Y>
where
    B: BufferableBulk,
    F: FnMut(&B::Item) -> Y,
    Y: Future<Output = bool>
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

    fn tasks(self: Pin<&mut Self>) -> Pin<&mut [Task<B::Item, Y>]>
    {
        unsafe { self.map_unchecked_mut(|this| this.tasks.as_mut_slice()) }
    }

    fn queue(self: Pin<&mut Self>) -> &mut B::IntoIter
    {
        unsafe { &mut self.get_unchecked_mut().queue }
    }

    fn push_task(self: Pin<&mut Self>, value: B::Item) -> Pin<&mut Task<B::Item, Y>>
    {
        let this = unsafe { self.get_unchecked_mut() };
        let mut task = unsafe { Pin::new_unchecked(this.tasks.push_mut(Task::new(value))) };
        task.as_mut().start(&mut this.action);
        task
    }

    fn cancel(self: Pin<&mut Self>)
    {
        for task in self.tasks().bulk_pin_mut()
        {
            task.cancel()
        }
    }
}
impl<B, F, Y> Future for FindAsync<B, F, Y>
where
    B: BufferableBulk,
    F: FnMut(&B::Item) -> Y,
    Y: Future<Output = bool>
{
    type Output = Option<B::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let mut ready = match self.as_mut().tasks().bulk_pin_mut().try_fold(true, |ready, mut task| {
            if !task.is_taken()
            {
                match task.as_mut().poll(cx)
                {
                    Poll::Pending => return Ok(false),
                    Poll::Ready(Some(found)) => return Err(found),
                    Poll::Ready(None) => ()
                }
            }

            Ok(ready)
        })
        {
            Err(found) =>
            {
                self.cancel();
                return Poll::Ready(Some(found));
            }
            Ok(ready) => ready
        };

        while let Some(value) = self.as_mut().queue().next()
        {
            let mut task = self.as_mut().push_task(value);
            if !task.is_taken()
            {
                match task.as_mut().poll(cx)
                {
                    Poll::Pending => ready = false,
                    Poll::Ready(Some(found)) => return Poll::Ready(Some(found)),
                    Poll::Ready(None) => ()
                }
            }
        }

        if ready { Poll::Ready(None) } else { Poll::Pending }
    }
}

struct Task<T, Y>
where
    Y: Future<Output = bool>
{
    item: Option<T>,
    future: MaybeDone<Y>
}
impl<T, Y> Task<T, Y>
where
    Y: Future<Output = bool>
{
    fn new(item: T) -> Self
    {
        Self {
            item: Some(item),
            future: MaybeDone::Taken
        }
    }

    fn item(self: Pin<&mut Self>) -> Option<Pin<&mut T>>
    {
        unsafe { self.map_unchecked_mut(|x| &mut x.item) }.as_pin_mut()
    }

    fn take_item(self: Pin<&mut Self>) -> Option<T>
    {
        unsafe { self.get_unchecked_mut() }.item.take()
    }

    fn future(self: Pin<&mut Self>) -> Pin<&mut MaybeDone<Y>>
    {
        unsafe { self.map_unchecked_mut(|x| &mut x.future) }
    }

    fn cancel(self: Pin<&mut Self>)
    {
        self.future().cancel();
    }

    fn start<F>(mut self: Pin<&mut Self>, action: &mut F)
    where
        F: FnMut(&T) -> Y
    {
        if let Some(item) = self.as_mut().item()
        {
            let future = action(&*item);
            self.future().restart(future);
        }
    }

    fn is_taken(&self) -> bool
    {
        self.future.is_taken()
    }
}
impl<T, Y> Future for Task<T, Y>
where
    Y: Future<Output = bool>
{
    type Output = Option<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        if self.as_mut().future().is_taken()
        {
            return Poll::Ready(None);
        }
        if self.as_mut().future().poll(cx).is_ready()
            && let Some(is_ready) = self.as_mut().future().take_output()
        {
            if let Some(item) = self.take_item()
                && is_ready
            {
                return Poll::Ready(Some(item));
            }
            else
            {
                return Poll::Ready(None);
            }
        }
        Poll::Pending
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
        let a = [1, 2, 4, 8, 16, 32, 64, 128];

        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

        rt.block_on(async {
            let res = a.into_bulk().find_async(async |n| *n == 8).await;
            assert_eq!(res, Some(8))
        })
    }
}
