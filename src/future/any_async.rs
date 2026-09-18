use core::{
    ops::ControlFlow,
    pin::Pin,
    task::{Context, Poll}
};

use crate::{TryForEachAsync, util::BufferableBulk};

pub struct AnyAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = bool>>
{
    future: TryForEachAsync<B, Functor<F>>
}
impl<B, F> AnyAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = bool>>
{
    pub(crate) fn new(bulk: B, condition: F) -> Self
    {
        Self {
            future: bulk.try_for_each_async(Functor { condition })
        }
    }
}
impl<B, F> Future for AnyAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = bool>>
{
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let future = unsafe { self.map_unchecked_mut(|x| &mut x.future) };
        future.poll(cx).map(|y| y.is_break())
    }
}

struct Visitor<Y>
where
    Y: Future<Output = bool>
{
    future: Y
}
impl<Y> Future for Visitor<Y>
where
    Y: Future<Output = bool>
{
    type Output = ControlFlow<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let future = unsafe { self.map_unchecked_mut(|x| &mut x.future) };
        future.poll(cx).map(|x| if x { ControlFlow::Break(()) } else { ControlFlow::Continue(()) })
    }
}

struct Functor<F>
{
    condition: F
}
impl<T, F, Y> FnOnce<(T,)> for Functor<F>
where
    F: FnMut(T) -> Y,
    Y: Future<Output = bool>
{
    type Output = Visitor<Y>;

    extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
    {
        self.call_mut(args)
    }
}
impl<T, F, Y> FnMut<(T,)> for Functor<F>
where
    F: FnMut(T) -> Y,
    Y: Future<Output = bool>
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        Visitor { future: (self.condition)(x) }
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn it_works()
    {
        let a = [64, 128, 256];

        tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
            let y = a.into_bulk().any_async(async |x| x < 0).await;
            assert!(!y)
        })
    }
}
