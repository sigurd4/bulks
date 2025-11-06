use core::{marker::Destruct, ops::Try};

use crate::{Bulk, DoubleEndedBulk, StaticBulk, util::{ArrayBuffer, Length, LengthSpec, LengthWindowed}};

/// A bulk over the mapped windows of another bulk.
///
/// This `struct` is created by the [`Bulk::map_windows`]. See its
/// documentation for more information.
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct MapWindows<I, F, const N: usize>
where
    I: Bulk,
    F: for<'a> FnMut<(&'a [I::Item; N],)>
{
    bulk: I,
    f: F,
}

impl<I: Bulk, F, U, const N: usize> MapWindows<I, F, N>
where
    I: Bulk,
    F: FnMut(&[I::Item; N]) -> U
{
    pub(crate) const fn new(bulk: I, f: F) -> Self
    {
        assert!(N != 0, "array in `Bulk::map_windows` must contain more than 0 elements");

        // Only ZST arrays' length can be so large.
        if core::mem::size_of::<I::Item>() != 0
        {
            assert!(
                N.checked_mul(2).is_some(),
                "array size of `Iterator::map_windows` is too large"
            );
        }

        Self {
            bulk,
            f
        }
    }
}

impl<I: Bulk, F, U, const N: usize> IntoIterator for MapWindows<I, F, N>
where
    I: Bulk,
    F: FnMut(&[I::Item; N]) -> U
{
    type Item = U;
    type IntoIter = core::iter::MapWindows<I::IntoIter, F, N>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter()
            .map_windows(f)
    }
}

impl<I: Bulk, F, U, const N: usize> const Bulk for MapWindows<I, F, N>
where
    I: ~const Bulk<Item: ~const Destruct>,
    F: ~const FnMut(&[I::Item; N]) -> U + ~const Destruct
{
    type MinLength<V> = <<<I::MinLength<V> as Length>::LengthSpec as LengthWindowed<N>>::LengthWindowed as LengthSpec>::Length<V>;
    type MaxLength<V> = <<<I::MaxLength<V> as Length>::LengthSpec as LengthWindowed<N>>::LengthWindowed as LengthSpec>::Length<V>;

    fn len(&self) -> usize
    {
        let Self { bulk, f: _ } = self;
        bulk.len().saturating_sub(N - 1)
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, f: _ } = self;
        bulk.len() > N - 1
    }

    fn for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: map } = self;
        bulk.for_each(Closure::<_, _, _, _, _, false> {
            map,
            f,
            buffer: ArrayBuffer::new()
        });
    }
    fn try_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, f: map } = self;
        bulk.try_for_each(TryClosure::<_, _, _, _, _, _, false> {
            map,
            f,
            buffer: ArrayBuffer::new()
        })
    }
}
impl<I: Bulk, F, U, const N: usize> const DoubleEndedBulk for MapWindows<I, F, N>
where
    I: ~const DoubleEndedBulk<Item: ~const Destruct>,
    F: ~const FnMut(&[I::Item; N]) -> U + ~const Destruct,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, f: map } = self;
        bulk.rev_for_each(Closure::<_, _, _, _, _, true> {
            map,
            f,
            buffer: ArrayBuffer::new()
        });
    }
    fn try_rev_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, f: map } = self;
        bulk.try_rev_for_each(TryClosure::<_, _, _, _, _, _, true> {
            map,
            f,
            buffer: ArrayBuffer::new()
        })
    }
}
unsafe impl<I: Bulk, F, T, U, const N: usize, const M: usize> StaticBulk for MapWindows<I, F, N>
where
    I: StaticBulk<Item = T>,
    F: FnMut(&[T; N]) -> U,
    Self: Bulk<MinLength<Self::Item> = [Self::Item; M], MaxLength<Self::Item> = [Self::Item; M]>
{
    type Array<W> = [W; M];
}

struct Closure<F, FF, T, U, const N: usize, const REV: bool>
where
    F: FnMut(&[T; N]) -> U,
    FF: FnMut(U)
{
    map: F,
    f: FF,
    buffer: ArrayBuffer<T, N, REV>
}
impl<F, FF, T, U, const N: usize, const REV: bool> const FnOnce<(T,)> for Closure<F, FF, T, U, N, REV>
where
    T: ~const Destruct,
    F: ~const FnMut(&[T; N]) -> U + ~const Destruct,
    FF: ~const FnMut(U) + ~const Destruct
{
    type Output = ();

    extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
    {
        self.call_mut(args)
    }
}
impl<F, FF, T, U, const N: usize, const REV: bool> const FnMut<(T,)> for Closure<F, FF, T, U, N, REV>
where
    T: ~const Destruct,
    F: ~const FnMut(&[T; N]) -> U,
    FF: ~const FnMut(U)
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let Self { map, f, buffer } = self;
        buffer.push_out(x);
        if let Some(window) = buffer.as_array()
        {
            f(map(window))
        }
    }
}

struct TryClosure<F, FF, T, U, R, const N: usize, const REV: bool>
where
    F: FnMut(&[T; N]) -> U,
    FF: FnMut(U) -> R
{
    map: F,
    f: FF,
    buffer: ArrayBuffer<T, N, REV>
}
impl<F, FF, T, U, R, const N: usize, const REV: bool> const FnOnce<(T,)> for TryClosure<F, FF, T, U, R, N, REV>
where
    T: ~const Destruct,
    F: ~const FnMut(&[T; N]) -> U + ~const Destruct,
    FF: ~const FnMut(U) -> R + ~const Destruct,
    R: ~const Try<Output = (), Residual: ~const Destruct>
{
    type Output = R;

    extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
    {
        self.call_mut(args)
    }
}
impl<F, FF, T, U, R, const N: usize, const REV: bool> const FnMut<(T,)> for TryClosure<F, FF, T, U, R, N, REV>
where
    T: ~const Destruct,
    F: ~const FnMut(&[T; N]) -> U,
    FF: ~const FnMut(U) -> R,
    R: ~const Try<Output = (), Residual: ~const Destruct>
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let Self { map, f, buffer } = self;
        buffer.push_out(x);
        if let Some(window) = buffer.as_array()
        {
            f(map(window))?
        }
        R::from_output(())
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let mut i = 0;
        let a = crate::repeat_n_with(|| {i += 1; i}, [(); 20])
            .map_windows(|&[n, m]| n + m)
            .collect::<[_; _]>();

        println!("{a:?}")
    }
}