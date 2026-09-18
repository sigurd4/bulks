use core::{
    ops::ControlFlow,
    pin::Pin,
    task::{Context, Poll}
};

use crate::{Enumerate, TryForEachAsync, util::BufferableBulk};

pub struct PositionAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = bool>>
{
    future: TryForEachAsync<Enumerate<B>, Functor<F>>
}
impl<B, F> PositionAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = bool>>
{
    pub(crate) fn new(bulk: B, condition: F) -> Self
    {
        Self {
            future: bulk.enumerate().try_for_each_async(Functor { condition })
        }
    }
}
impl<B, F> Future for PositionAsync<B, F>
where
    B: BufferableBulk,
    F: FnMut<(B::Item,), Output: Future<Output = bool>>
{
    type Output = Option<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let future = unsafe { self.map_unchecked_mut(|x| &mut x.future) };
        future.poll(cx).map(|y| match y
        {
            ControlFlow::Break(i) => Some(i),
            ControlFlow::Continue(()) => None
        })
    }
}

struct Visitor<Y>
where
    Y: Future<Output = bool>
{
    index: usize,
    future: Y
}
impl<Y> Future for Visitor<Y>
where
    Y: Future<Output = bool>
{
    type Output = ControlFlow<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let i = self.index;
        let future = unsafe { self.map_unchecked_mut(|x| &mut x.future) };
        future.poll(cx).map(|x| if x { ControlFlow::Break(i) } else { ControlFlow::Continue(()) })
    }
}

struct Functor<F>
{
    condition: F
}
impl<T, F, Y> FnOnce<((usize, T),)> for Functor<F>
where
    F: FnMut(T) -> Y,
    Y: Future<Output = bool>
{
    type Output = Visitor<Y>;

    extern "rust-call" fn call_once(mut self, args: ((usize, T),)) -> Self::Output
    {
        self.call_mut(args)
    }
}
impl<T, F, Y> FnMut<((usize, T),)> for Functor<F>
where
    F: FnMut(T) -> Y,
    Y: Future<Output = bool>
{
    extern "rust-call" fn call_mut(&mut self, ((i, x),): ((usize, T),)) -> Self::Output
    {
        Visitor {
            index: i,
            future: (self.condition)(x)
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
        let a = [64, 128, 256];

        tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
            let y = a.into_bulk().position_async(async |x| x > 128).await;
            assert_eq!(y, Some(2))
        })
    }
}
