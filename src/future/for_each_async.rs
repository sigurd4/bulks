use core::{
    pin::Pin,
    task::{Context, Poll}
};

use array_trait::AsSlice;

use crate::{
    AsBulkMut, Bulk, BulkLength, util::{Buffer, BufferableBulk, MaybeDone}
};

pub struct ForEachAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future>
{
    queue: B::IntoIter,
    tasks: Buffer<MaybeDone<F::Output>, BulkLength<B>>,
    action: F
}
impl<B, F> ForEachAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future>
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

    fn queue(self: Pin<&mut Self>) -> &mut B::IntoIter
    {
        unsafe { &mut self.get_unchecked_mut().queue }
    }

    fn push_task(self: Pin<&mut Self>, value: B::Item) -> Pin<&mut MaybeDone<F::Output>>
    {
        unsafe { self.map_unchecked_mut(|this| this.tasks.push_mut(MaybeDone::Future((this.action)(value)))) }
    }
}

impl<B, F> Future for ForEachAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future>
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let mut ready = self.as_mut().tasks().bulk_pin_mut().fold(true, |ready, task| ready & task.poll(cx).is_ready());

        while let Some(value) = self.as_mut().queue().next()
        {
            let task = self.as_mut().push_task(value);
            ready &= task.poll(cx).is_ready()
        }

        if ready { Poll::Ready(()) } else { Poll::Pending }
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [256, 512, 1024];

        tokio_test::block_on(async {
            a.into_bulk()
                .for_each_async(async |n| {
                    let m = crate::repeat_n(1, n).sum_from(0);
                    assert_eq!(n, m);
                    println!("{m}");
                })
                .await
        })
    }
}
