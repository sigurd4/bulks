use core::{marker::Destruct, ops::{ControlFlow, FromResidual, Residual, Try}};

use crate::{util::{self, ArrayBuffer}, Bulk, DoubleEndedBulk, IntoBulk, Rev, StaticBulk};

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

    const fn skip_len<const REV: bool>(&self) -> usize
    where
        I: ~const Bulk
    {
        if REV
        {
            self.bulk.len() % N
        }
        else
        {
            0
        }
    }

    const fn for_each_closure<'a, F, const REV: bool>(&self, f: F, remainder: &'a mut ArrayBuffer<I::Item, N, REV>) -> impl ~const FnMut(I::Item) + ~const Destruct + 'a
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

        Closure {
            f,
            buffer: remainder,
            skip: self.skip_len::<REV>()
        }
    }
    
    const fn try_for_each_closure<'a, F, R, const REV: bool>(&self, f: F, remainder: &'a mut ArrayBuffer<I::Item, N, REV>) -> impl ~const FnMut(I::Item) -> R + ~const Destruct + 'a
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

        Closure {
            f,
            buffer: remainder,
            skip: self.skip_len::<REV>()
        }
    }

    pub const fn for_each_with_remainder<F>(self, f: F) -> <ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk
    where
        Self: Sized,
        I: ~const Bulk<Item: ~const Destruct>,
        F: ~const FnMut(<Self as IntoIterator>::Item) + ~const Destruct,
        ArrayBuffer<I::Item, N, false>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let closure = self.for_each_closure(f, &mut remainder);
        let Self { bulk } = self;
        bulk.for_each(closure);
        remainder.into_bulk()
    }

    pub const fn try_for_each_with_remainder<F, R, RR>(self, f: F) -> RR
    where
        Self: Sized,
        I: ~const Bulk<Item: ~const Destruct>,
        F: ~const FnMut(<Self as IntoIterator>::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct + Residual<<ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk, TryType = RR>>,
        RR: ~const Try<Output = <ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk, Residual = R::Residual>,
        ArrayBuffer<I::Item, N, false>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let closure = self.try_for_each_closure(f, &mut remainder);
        let Self { bulk } = self;
        match bulk.try_for_each(closure).branch()
        {
            ControlFlow::Break(residual) => RR::from_residual(residual),
            ControlFlow::Continue(()) => RR::from_output(remainder.into_bulk())
        }
    }

    const fn rev_for_each_with_remainder<F>(self, f: F) -> <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk
    where
        Self: Sized,
        I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
        F: ~const FnMut(<Self as IntoIterator>::Item) + ~const Destruct,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let closure = self.for_each_closure(f, &mut remainder);
        let Self { bulk } = self;
        bulk.rev_for_each(closure);
        remainder.into_bulk()
    }

    const fn try_rev_for_each_with_remainder<F, R, RR>(self, f: F) -> RR
    where
        Self: Sized,
        I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
        F: ~const FnMut(<Self as IntoIterator>::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct + Residual<<ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk, TryType = RR>>,
        RR: ~const Try<Output = <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk, Residual = R::Residual>,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let closure = self.try_for_each_closure(f, &mut remainder);
        let Self { bulk } = self;
        match bulk.try_rev_for_each(closure).branch()
        {
            ControlFlow::Break(residual) => RR::from_residual(residual),
            ControlFlow::Continue(()) => RR::from_output(remainder.into_bulk())
        }
    }

    pub const fn collect_array_with_remainder(self) -> (<Self as StaticBulk>::Array<<Self as IntoIterator>::Item>, <ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk)
    where
        Self: StaticBulk<Item: ~const Destruct>,
        I: ~const Bulk<Item: ~const Destruct>,
        ArrayBuffer<I::Item, N, false>: ~const IntoBulk
    {
        let remainder;
        let array = util::collect_array_with!(|f| {
            remainder = self.for_each_with_remainder(f)
        }; for Self);
        (array, remainder)
    }

    pub const fn try_collect_array_with_remainder(self) -> <<<Self as IntoIterator>::Item as Try>::Residual as Residual<(<Self as StaticBulk>::Array<<<Self as IntoIterator>::Item as Try>::Output>, <ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk)>>::TryType
    where
        Self: StaticBulk<Item: ~const Destruct + ~const Try<
            Residual:
                Residual<(), TryType: ~const Try>
                + Residual<(<Self as StaticBulk>::Array<<<Self as IntoIterator>::Item as Try>::Output>, <ArrayBuffer<I::Item, N, false> as IntoBulk>::IntoBulk), TryType: ~const Try>
                + ~const Destruct,
            Output: ~const Destruct
        >>,
        I: ~const Bulk<Item: ~const Destruct>,
        ArrayBuffer<I::Item, N, false>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let array = util::try_collect_array_with!(|f| {
            let closure = self.try_for_each_closure(f, &mut remainder);
            let Self { bulk } = self;
            match bulk.try_for_each(closure).branch()
            {
                ControlFlow::Break(residual) => return FromResidual::from_residual(residual),
                ControlFlow::Continue(()) => ()
            }
        }; for Self);
        Try::from_output((array, remainder.into_bulk()))
    }

    const fn rev_collect_array_with_remainder(self) -> (<Self as StaticBulk>::Array<<Self as IntoIterator>::Item>, <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk)
    where
        Self: StaticBulk<Item: ~const Destruct>,
        I: ~const Bulk<Item: ~const Destruct> + ~const DoubleEndedBulk,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        let remainder;
        let array = util::collect_array_with!(|f| {
            remainder = self.rev_for_each_with_remainder(f)
        }; for Self);
        (array, remainder)
    }

    const fn try_rev_collect_array_with_remainder(self) -> <<<Self as IntoIterator>::Item as Try>::Residual as Residual<(<Self as StaticBulk>::Array<<<Self as IntoIterator>::Item as Try>::Output>, <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk)>>::TryType
    where
        Self: StaticBulk<Item: ~const Destruct + ~const Try<
            Residual:
                Residual<(), TryType: ~const Try>
                + Residual<(<Self as StaticBulk>::Array<<<Self as IntoIterator>::Item as Try>::Output>, <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk), TryType: ~const Try>
                + ~const Destruct,
            Output: ~const Destruct
        >>,
        I: ~const Bulk<Item: ~const Destruct> + ~const DoubleEndedBulk,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        let mut remainder = ArrayBuffer::new();
        let array = util::try_collect_array_with!(|f| {
            let closure = self.try_for_each_closure(f, &mut remainder);
            let Self { bulk } = self;
            match bulk.try_rev_for_each(closure).branch()
            {
                ControlFlow::Break(residual) => return FromResidual::from_residual(residual),
                ControlFlow::Continue(()) => ()
            }
        }; for Self);
        Try::from_output((array, remainder.into_bulk()))
    }
}

impl<I, const N: usize> Rev<ArrayChunks<I, N>>
where
    I: DoubleEndedBulk
{
    pub const fn for_each_with_remainder<F>(self, f: F) -> <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk
    where
        ArrayChunks<I, N>: Sized,
        I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
        F: ~const FnMut(<Self as IntoIterator>::Item) + ~const Destruct,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        self.into_inner().rev_for_each_with_remainder(f)
    }

    pub const fn try_for_each_with_remainder<F, R, RR>(self, f: F) -> RR
    where
        ArrayChunks<I, N>: Sized,
        I: ~const DoubleEndedBulk<Item: ~const Destruct> + ~const Bulk,
        F: ~const FnMut(<ArrayChunks<I, N> as IntoIterator>::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct + Residual<<ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk, TryType = RR>>,
        RR: ~const Try<Output = <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk, Residual = R::Residual>,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        self.into_inner().try_rev_for_each_with_remainder(f)
    }

    pub const fn collect_array_with_remainder(self) -> (<ArrayChunks<I, N> as StaticBulk>::Array<<ArrayChunks<I, N> as IntoIterator>::Item>, <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk)
    where
        ArrayChunks<I, N>: StaticBulk<Item: ~const Destruct>,
        I: ~const Bulk<Item: ~const Destruct> + ~const DoubleEndedBulk,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        self.into_inner().rev_collect_array_with_remainder()
    }

    pub const fn try_collect_array_with_remainder(self) -> <<<ArrayChunks<I, N> as IntoIterator>::Item as Try>::Residual as Residual<(<ArrayChunks<I, N> as StaticBulk>::Array<<<ArrayChunks<I, N> as IntoIterator>::Item as Try>::Output>, <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk)>>::TryType
    where
        ArrayChunks<I, N>: StaticBulk<Item: ~const Destruct + ~const Try<
            Residual:
                Residual<(), TryType: ~const Try>
                + Residual<(<ArrayChunks<I, N> as StaticBulk>::Array<<<ArrayChunks<I, N> as IntoIterator>::Item as Try>::Output>, <ArrayBuffer<I::Item, N, true> as IntoBulk>::IntoBulk), TryType: ~const Try>
                + ~const Destruct,
            Output: ~const Destruct
        >>,
        I: ~const Bulk<Item: ~const Destruct> + ~const DoubleEndedBulk,
        ArrayBuffer<I::Item, N, true>: ~const IntoBulk
    {
        self.into_inner().try_rev_collect_array_with_remainder()
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

        let a = [1, 2, 3, 4, 5, 6];
        let (b, r) = a.into_bulk().array_chunks::<4>().collect_array_with_remainder();
        let r = r.collect::<Vec<_>>();

        println!("b = {b:?}");
        println!("r = {r:?}");
    }
}