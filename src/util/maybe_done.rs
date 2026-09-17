use core::{
    future::Future,
    ops::{ControlFlow, Try},
    pin::Pin,
    task::{Context, Poll}
};

pub(crate) enum MaybeDone<F: Future>
{
    Future(F),
    Done(F::Output),
    Taken
}

impl<F: Future> MaybeDone<F>
{
    pub fn take_residual(mut self: Pin<&mut Self>) -> Option<<F::Output as Try>::Residual>
    where
        F::Output: Try
    {
        self.as_mut().take_output().and_then(|output| match output.branch()
        {
            ControlFlow::Break(residual) => Some(residual),
            ControlFlow::Continue(output) =>
            {
                core::mem::replace(unsafe { self.get_unchecked_mut() }, Self::Done(Try::from_output(output)));
                None
            }
        })
    }

    pub fn take_output(self: Pin<&mut Self>) -> Option<F::Output>
    {
        unsafe { self.get_unchecked_mut() }._take_output()
    }

    fn _take_output(&mut self) -> Option<F::Output>
    {
        match self
        {
            MaybeDone::Done(_) => match core::mem::replace(self, Self::Taken)
            {
                MaybeDone::Done(val) => Some(val),
                _ => unreachable!()
            },
            _ => None
        }
    }

    pub fn into_output(mut self) -> Option<F::Output>
    {
        self._take_output()
    }

    pub fn restart(self: Pin<&mut Self>, future: F)
    {
        *unsafe { self.get_unchecked_mut() } = Self::Future(future)
    }

    pub fn cancel(self: Pin<&mut Self>)
    {
        *unsafe { self.get_unchecked_mut() } = Self::Taken
    }

    pub fn is_taken(&self) -> bool
    {
        core::matches!(self, MaybeDone::Taken)
    }
}

impl<F: Future> Future for MaybeDone<F>
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        // SAFETY: pinning in structural for `f`
        unsafe {
            match *self.as_mut().get_unchecked_mut()
            {
                MaybeDone::Future(ref mut f) =>
                {
                    let val = core::task::ready!(Pin::new_unchecked(f).poll(cx));
                    self.set(Self::Done(val));
                }
                MaybeDone::Done(_) =>
                {}
                MaybeDone::Taken => unreachable!()
            }
        }

        Poll::Ready(())
    }
}
