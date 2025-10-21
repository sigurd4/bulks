use core::{marker::Destruct, ops::Try};

use crate::{util::ArrayBuffer, Bulk, DoubleEndedBulk, StaticBulk};

/// A bulk over `N` elements of the bulk at a time.
///
/// The chunks do not overlap. If `N` does not divide the length of the
/// iterator, then the last up to `N-1` elements will be omitted.
///
/// This `struct` is created by the [`array_chunks`][Bulk::array_chunks]
/// method on [`Bulk`]. See its documentation for more.
#[derive(Debug, Clone)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct ArrayChunks<I, const N: usize>
where
    I: Bulk
{
    bulk: I
}

impl<I, const N: usize> ArrayChunks<I, N>
where
    I: Bulk
{
    #[track_caller]
    pub(crate) const fn new(bulk: I) -> Self
    {
        assert!(N != 0, "chunk size must be non-zero");
        Self {
            bulk
        }
    }
}

impl<I, const N: usize> IntoIterator for ArrayChunks<I, N>
where
    I: Bulk
{
    type Item = [I::Item; N];
    type IntoIter = core::iter::ArrayChunks<I::IntoIter, N>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self {bulk} = self;
        bulk.into_iter().array_chunks()
    }
}
impl<I, const N: usize> const Bulk for ArrayChunks<I, N>
where
    I: ~const Bulk<Item: ~const Destruct>,
{
    #[inline]
    fn len(&self) -> usize
    {
        let Self {bulk} = self;
        bulk.len()/N
    }
    
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<T, F, const N: usize>
        where
            F: FnMut([T; N])
        {
            f: F,
            buffer: ArrayBuffer<T, N, false>
        }

        impl<T, F, const N: usize> const FnOnce<(T,)> for Closure<T, F, N>
        where
            F: ~const FnMut([T; N]) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
            {
                self.call_mut(args)
            }
        }
        impl<T, F, const N: usize> const FnMut<(T,)> for Closure<T, F, N>
        where
            F: ~const FnMut([T; N]) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                if let Some(array) = self.buffer.push_take_array(x)
                {
                    (self.f)(array)
                }
            }
        }

        let Self {bulk} = self;
        bulk.for_each(Closure {
            f,
            buffer: ArrayBuffer::new()
        })
    }
    
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<T, F, R, const N: usize>
        where
            F: FnMut([T; N]) -> R,
            R: Try<Output = ()>
        {
            f: F,
            buffer: ArrayBuffer<T, N, false>
        }

        impl<T, F, R, const N: usize> const FnOnce<(T,)> for Closure<T, F, R, N>
        where
            F: ~const FnMut([T; N]) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
            {
                self.call_mut(args)
            }
        }
        impl<T, F, R, const N: usize> const FnMut<(T,)> for Closure<T, F, R, N>
        where
            F: ~const FnMut([T; N]) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                if let Some(array) = self.buffer.push_take_array(x)
                {
                    (self.f)(array)?
                }
                R::from_output(())
            }
        }

        let Self {bulk} = self;
        bulk.try_for_each(Closure {
            f,
            buffer: ArrayBuffer::new()
        })
    }
}
impl<I, const N: usize> const DoubleEndedBulk for ArrayChunks<I, N>
where
    I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<T, F, const N: usize>
        where
            F: FnMut([T; N])
        {
            f: F,
            buffer: ArrayBuffer<T, N, true>,
            skip: usize
        }

        impl<T, F, const N: usize> const FnOnce<(T,)> for Closure<T, F, N>
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
        impl<T, F, const N: usize> const FnMut<(T,)> for Closure<T, F, N>
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

        let Self {bulk} = self;
        let skip = bulk.len() % N;
        bulk.rev_for_each(Closure {
            f,
            buffer: ArrayBuffer::new(),
            skip
        })
    }
    
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<T, F, R, const N: usize>
        where
            F: FnMut([T; N]) -> R,
            R: Try<Output = ()>
        {
            f: F,
            buffer: ArrayBuffer<T, N, true>,
            skip: usize
        }

        impl<T, F, R, const N: usize> const FnOnce<(T,)> for Closure<T, F, R, N>
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
        impl<T, F, R, const N: usize> const FnMut<(T,)> for Closure<T, F, R, N>
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

        let Self {bulk} = self;
        let skip = bulk.len() % N;
        bulk.try_rev_for_each(Closure {
            f,
            buffer: ArrayBuffer::new(),
            skip
        })
    }
}
unsafe impl<I, T, const N: usize, const M: usize> StaticBulk for ArrayChunks<I, N>
where
    I: StaticBulk<Item = T, Array<T> = [T; M]>,
    [(); M/N]:
{
    type Array<U> = [U; M/N];
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let b = const {
            let a = [1, 2, 3, 4, 5, 6];
            a.into_bulk().array_chunks::<2>().rev().enumerate().collect::<[_; _]>()
        };

        println!("{b:?}");

        let c = b.into_bulk()
            .map(|(_, b)| b.into_bulk()
                .map(|b: u32| b.checked_sub(3))
                .collect::<Option<[_; _]>>()
            ).collect::<[_; _]>();

        println!("{c:?}");
    }
}