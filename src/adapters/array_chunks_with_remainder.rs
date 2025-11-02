use core::{marker::Destruct, ops::Try};

use crate::{util::ArrayBuffer, ArrayChunks, Bulk, StaticBulk};

#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct ArrayChunksWithRemainder<'a, I, const N: usize, const REV: bool>
where
    I: Bulk
{
    bulk: ArrayChunks<I, N>,
    remainder: &'a mut ArrayBuffer<I::Item, N, REV>
}

impl<'a, I, const N: usize, const REV: bool> ArrayChunksWithRemainder<'a, I, N, REV>
where
    I: Bulk
{
    #[track_caller]
    pub(crate) const fn new(bulk: I, remainder: &'a mut ArrayBuffer<I::Item, N, REV>) -> Self
    {
        Self {
            bulk: ArrayChunks::new(bulk),
            remainder
        }
    }

    const fn skip_len(&self) -> usize
    where
        I: ~const Bulk
    {
        let Self { bulk, remainder: _ } = self;
        bulk.skip_len::<REV>()
    }

    const fn for_each_closure<F>(self, f: F) -> (I, impl ~const FnMut(I::Item) + ~const Destruct + 'a)
    where
        Self: Sized,
        I: ~const Bulk<Item: ~const Destruct>,
        F: ~const FnMut(<Self as IntoIterator>::Item) + ~const Destruct + 'a
    {
        struct Closure<'a, T, F, const N: usize, const REV: bool>
        where
            F: FnMut([T; N])
        {
            f: F,
            buffer: &'a mut ArrayBuffer<T, N, REV>,
            skip: usize
        }

        impl<'a, T, F, const N: usize, const REV: bool> const FnOnce<(T,)> for Closure<'a, T, F, N, REV>
        where
            T: ~const Destruct,
            F: ~const FnMut([T; N]) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
            {
                self.call_mut(args)
            }
        }
        impl<'a, T, F, const N: usize, const REV: bool> const FnMut<(T,)> for Closure<'a, T, F, N, REV>
        where
            T: ~const Destruct,
            F: ~const FnMut([T; N]) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                if self.skip > 0
                {
                    self.skip -= 1
                }
                else if let Some(array) = self.buffer.push_take_array(x)
                {
                    (self.f)(array)
                }
            }
        }

        let skip = self.skip_len();
        let Self { bulk, remainder } = self;
        (
            bulk.into_inner(),
            Closure {
                f,
                buffer: remainder,
                skip
            }
        )
    }
    
    const fn try_for_each_closure<F, R>(self, f: F) -> (I, impl ~const FnMut(I::Item) -> R + ~const Destruct + 'a)
    where
        Self: Sized,
        <Self as IntoIterator>::Item: ~const Destruct,
        I: ~const Bulk<Item: ~const Destruct>,
        F: ~const FnMut(<Self as IntoIterator>::Item) -> R + ~const Destruct + 'a,
        R: ~const Try<Output = (), Residual: ~const Destruct> + 'a
    {
        struct Closure<'a, T, F, R, const N: usize, const REV: bool>
        where
            F: FnMut([T; N]) -> R,
            R: Try<Output = ()>
        {
            f: F,
            buffer: &'a mut ArrayBuffer<T, N, REV>,
            skip: usize
        }

        impl<'a, T, F, R, const N: usize, const REV: bool> const FnOnce<(T,)> for Closure<'a, T, F, R, N, REV>
        where
            T: ~const Destruct,
            F: ~const FnMut([T; N]) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
            {
                self.call_mut(args)
            }
        }
        impl<'a, T, F, R, const N: usize, const REV: bool> const FnMut<(T,)> for Closure<'a, T, F, R, N, REV>
        where
            T: ~const Destruct,
            F: ~const FnMut([T; N]) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                if self.skip > 0
                {
                    self.skip -= 1
                }
                else if let Some(array) = self.buffer.push_take_array(x)
                {
                    (self.f)(array)?
                }
                R::from_output(())
            }
        }

        let skip = self.skip_len();
        let Self { bulk, remainder } = self;
        (
            bulk.into_inner(),
            Closure {
                f,
                buffer: remainder,
                skip
            }
        )
    }
}

impl<'a, I, const N: usize, const REV: bool> IntoIterator for ArrayChunksWithRemainder<'a, I, N, REV>
where
    I: Bulk
{
    type Item = <ArrayChunks<I, N> as IntoIterator>::Item;
    type IntoIter = <ArrayChunks<I, N> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, remainder: _ } = self;
        bulk.into_iter()
    }
}
impl<'a, I, const N: usize, const REV: bool> const Bulk for ArrayChunksWithRemainder<'a, I, N, REV>
where
    I: ~const Bulk<Item: ~const Destruct>,
{
    #[inline]
    fn len(&self) -> usize
    {
        let Self { bulk, remainder: _ } = self;
        bulk.len()
    }
    
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let (bulk, closure) = self.for_each_closure(f);
        bulk.for_each(closure)
    }
    
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let (bulk, closure) = self.try_for_each_closure(f);
        bulk.try_for_each(closure)
    }
}
unsafe impl<'a, I, const N: usize, const REV: bool> StaticBulk for ArrayChunksWithRemainder<'a, I, N, REV>
where
    I: Bulk,
    ArrayChunks<I, N>: StaticBulk
{
    type Array<U> = <ArrayChunks<I, N> as StaticBulk>::Array<U>;
}