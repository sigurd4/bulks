use core::{
    ops::ControlFlow,
    pin::Pin,
    task::{Context, Poll}
};

use crate::{TryForEachAsync, util::BufferableBulk};

pub struct FindMapAsync<B, U, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = Option<U>>>
{
    future: TryForEachAsync<B, Functor<F>>
}
impl<B, U, F> FindMapAsync<B, U, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = Option<U>>>
{
    pub(crate) fn new(bulk: B, condition: F) -> Self
    {
        Self {
            future: bulk.try_for_each_async(Functor { condition })
        }
    }
}
impl<B, U, F> Future for FindMapAsync<B, U, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = Option<U>>>
{
    type Output = Option<U>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let future = unsafe { self.map_unchecked_mut(|x| &mut x.future) };
        future.poll(cx).map(|y| match y
        {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(found) => Some(found)
        })
    }
}

struct Visitor<U, Y>
where
    Y: Future<Output = Option<U>>
{
    future: Y
}
impl<U, Y> Future for Visitor<U, Y>
where
    Y: Future<Output = Option<U>>
{
    type Output = ControlFlow<U>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let future = unsafe { self.map_unchecked_mut(|x| &mut x.future) };
        future.poll(cx).map(|x| match x
        {
            Some(x) => ControlFlow::Break(x),
            None => ControlFlow::Continue(())
        })
    }
}

struct Functor<F>
{
    condition: F
}
impl<T, U, F, Y> FnOnce<(T,)> for Functor<F>
where
    F: FnMut(T) -> Y,
    Y: Future<Output = Option<U>>
{
    type Output = Visitor<U, Y>;

    extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
    {
        self.call_mut(args)
    }
}
impl<T, U, F, Y> FnMut<(T,)> for Functor<F>
where
    F: FnMut(T) -> Y,
    Y: Future<Output = Option<U>>
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
        let a = [8, 16, 32, 64] as [u32; 4];

        tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
            let y = a.into_bulk().find_map_async(async |x| x.checked_sub(64)).await;
            assert_eq!(y, Some(0))
        })
    }
}
