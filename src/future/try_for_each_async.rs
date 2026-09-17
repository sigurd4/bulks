use core::{
    ops::{FromResidual, Residual, Try},
    pin::Pin,
    task::{Context, Poll}
};

use array_trait::AsSlice;

use crate::{
    AsBulkMut, Bulk, BulkLength,
    util::{Buffer, BufferableBulk, MaybeDone}
};

pub struct TryForEachAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output: Try<Output = ()>>>
{
    queue: B::IntoIter,
    tasks: Buffer<MaybeDone<F::Output>, BulkLength<B>>,
    action: F
}
impl<B, F> TryForEachAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output: Try<Output = ()>>>
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

    fn cancel(self: Pin<&mut Self>)
    {
        for task in unsafe { self.get_unchecked_mut() }.tasks.bulk_mut()
        {
            task.cancel()
        }
    }
}

impl<B, F> Future for TryForEachAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output: Try<Output = ()>>>
{
    type Output = <<<<F as FnOnce<(B::Item,)>>::Output as Future>::Output as Try>::Residual as Residual<()>>::TryType;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let mut ready = match self.as_mut().tasks().bulk_pin_mut().try_fold(true, |ready, mut task| {
            if !task.is_taken()
            {
                if !task.as_mut().poll(cx).is_ready()
                {
                    return Ok(false);
                }

                if let Some(residual) = unsafe { task.get_unchecked_mut() }.take_residual()
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
            else if let Some(residual) = unsafe { task.get_unchecked_mut() }.take_residual()
            {
                self.cancel();
                return Poll::Ready(FromResidual::from_residual(residual));
            }
        }

        if ready { Poll::Ready(Try::from_output(())) } else { Poll::Pending }
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
        let a = ["1024", "512", "256", "lol"];

        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

        rt.block_on(async {
            let res = a
                .into_bulk()
                .try_for_each_async(async |n| {
                    let n = n.parse::<u64>().map_err(|_| n)?;
                    tokio::time::sleep(Duration::from_millis(n)).await;

                    println!("{n}");
                    panic!("Tasks will be cancelled before this point.");

                    Ok(())
                })
                .await;
            assert_eq!(res, Err("lol"))
        })
    }
}
