use core::{
    ops::{ControlFlow, FromResidual, Residual, Try},
    pin::Pin,
    task::{Context, Poll}
};

use array_trait::AsSlice;

use crate::{
    AsBulkMut, BulkLength,
    util::{Buffer, BufferableBulk, MaybeDone}
};

pub struct TryReduceAsync<B, F, R>
where
    B: BufferableBulk,
    F: FnMut<(B::Item, B::Item), Output: Future<Output = R>>,
    R: Try<Output = B::Item, Residual: Residual<Option<B::Item>>>
{
    queue: B::IntoIter,
    tasks: Buffer<MaybeDone<F::Output>, BulkLength<B>>,
    action: F
}
impl<B, F, R> TryReduceAsync<B, F, R>
where
    B: BufferableBulk,
    F: FnMut<(B::Item, B::Item), Output: Future<Output = R>>,
    R: Try<Output = B::Item, Residual: Residual<Option<B::Item>>>
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
            unsafe { self.map_unchecked_mut(|this| this.tasks.push_mut(MaybeDone::Done(R::from_output(lhs)))) }
        }
    }

    fn cancel(self: Pin<&mut Self>)
    {
        for task in self.tasks().bulk_pin_mut()
        {
            task.cancel()
        }
    }
}

impl<B, F, R> Future for TryReduceAsync<B, F, R>
where
    B: BufferableBulk,
    F: FnMut<(B::Item, B::Item), Output: Future<Output = R>>,
    R: Try<Output = B::Item, Residual: Residual<Option<B::Item>>>
{
    type Output = <R::Residual as Residual<Option<B::Item>>>::TryType;

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
                            let lhs = match self.as_mut().task(j).take_output().unwrap().branch()
                            {
                                ControlFlow::Continue(lhs) => lhs,
                                ControlFlow::Break(residual) =>
                                {
                                    self.cancel();
                                    return Poll::Ready(FromResidual::from_residual(residual));
                                }
                            };
                            let rhs = match self.as_mut().task(i).take_output().unwrap().branch()
                            {
                                ControlFlow::Continue(lhs) => lhs,
                                ControlFlow::Break(residual) =>
                                {
                                    self.cancel();
                                    return Poll::Ready(FromResidual::from_residual(residual));
                                }
                            };
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
                Some(j) => match self.as_mut().task(j).take_output().map(Try::branch)
                {
                    Some(ControlFlow::Continue(output)) => Poll::Ready(Try::from_output(Some(output))),
                    Some(ControlFlow::Break(residual)) =>
                    {
                        self.cancel();
                        Poll::Ready(FromResidual::from_residual(residual))
                    }
                    None => Poll::Ready(Try::from_output(None))
                },
                None => Poll::Ready(Try::from_output(None))
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
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3];

        tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
            let n = a.into_bulk().try_reduce_async(async |a, b| u32::checked_add(a, b)).await;

            assert_eq!(n, Some(Some(1 + 2 + 3)))
        })
    }
}
