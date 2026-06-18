use core::{marker::Destruct, ops::Try, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};

use crate::{Bulk, DoubleEndedBulk, IntoBulk, IntoContained, SplitBulk};

pub const fn resize_with<I, F, L>(iterable: I, n: L, f: F) -> ResizeWith<
    <<I as IntoContained>::IntoContained as IntoBulk>::IntoBulk,
    F,
    L::Length<()>
>
where
    I: ~const IntoContained,
    L: LengthValue,
    F: FnMut() -> I::Item
{
    unsafe {
        ResizeWith::new(iterable.into_contained().into_bulk(), n, f)
    }
}

/// A bulk that only delivers exactly `n` elements, taking the first at most `n` elements of `bulk`, then calls `f` to produce more.
///
/// This `struct` is created by the [`resize_with`](Bulk::resize_with) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct ResizeWith<T, F = fn() -> <T as IntoIterator>::Item, N = [()]>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized,
    F: FnMut() -> T::Item
{
    bulk: T,
    n: <N as Pointee>::Metadata,
    padder: F
}

impl<T, F, N> ResizeWith<T, F, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized,
    F: FnMut() -> T::Item
{
    pub(crate) const fn new(bulk: T, n: N::Value, padder: F) -> ResizeWith<T, F, N>
    {
        Self { bulk, n: length::value::into_metadata(n), padder }
    }
}
impl<T, F, N> IntoIterator for ResizeWith<T, F, N>
where
    T: Bulk,
    N: Length<Elem = ()> + ?Sized,
    F: FnMut() -> T::Item
{
    type Item = T::Item;
    type IntoIter = <<core::iter::Take<core::iter::Chain<T::IntoIter, core::iter::RepeatWith<F>>> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, n, padder } = self;
        unsafe {
            bulk.into_iter()
                .chain(core::iter::repeat_with(padder))
                .take(length::len_metadata::<N>(n))
                .into_contained()
                .into_iter()
        }
    }
}
impl<T, F, N> const Bulk for ResizeWith<T, F, N>
where
    T: ~const Bulk<Item: Copy + ~const Destruct>,
    N: Length<Elem = ()> + ?Sized,
    F: ~const FnMut() -> T::Item + ~const Destruct
{
    type Length = N;
    type MinLength = N;
    type MaxLength = N;

    fn len(&self) -> usize
    {
        let Self { bulk: _, n, padder: _ } = self;
        length::len_metadata::<N>(*n)
    }
    fn for_each<FF>(self, mut f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, n, mut padder } = self;
        let mut m = bulk.len();
        bulk.take(length::value::from_metadata::<N::Value>(n))
            .for_each(&mut f);
        while m < length::len_metadata::<N>(n)
        {
            f(padder());
            m += 1
        }
    }
    fn try_for_each<FF, R>(self, mut f: FF) -> R
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, n, mut padder } = self;
        let mut m = bulk.len();
        bulk.take(length::value::from_metadata::<N::Value>(n))
            .try_for_each(&mut f)?;
        while m < length::len_metadata::<N>(n)
        {
            f(padder())?;
            m += 1
        }
        R::from_output(())
    }
}
impl<T, F, N> const DoubleEndedBulk for ResizeWith<T, F, N>
where
    T: ~const DoubleEndedBulk<Item: Copy + ~const Destruct> + ~const Bulk + ~const Destruct,
    N: Length<Elem = ()> + ?Sized,
    F: ~const Fn() -> T::Item + ~const Destruct,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<FF>(self, mut f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, n, padder } = self;
        let mut m = bulk.len();
        while m < length::len_metadata::<N>(n)
        {
            f(padder());
            m += 1
        }
        let m = bulk.length();
        bulk.rev()
            .skip(length::value::saturating_sub(m, length::value::from_metadata::<N::Value>(n)))
            .take(length::value::from_metadata::<N::Value>(n))
            .for_each(f);
    }
    fn try_rev_for_each<FF, R>(self, mut f: FF) -> R
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, n, padder } = self;
        let mut m = bulk.len();
        while m < length::len_metadata::<N>(n)
        {
            f(padder())?;
            m += 1
        }
        let m = bulk.length();
        bulk.rev()
            .skip(length::value::saturating_sub(m, length::value::from_metadata::<N::Value>(n)))
            .take(length::value::from_metadata::<N::Value>(n))
            .try_for_each(f)
    }
}
impl<T, F, N, NN, M, R> const SplitBulk<M> for ResizeWith<T, F, N>
where
    T: ~const SplitBulk<M, Item: Copy + ~const Destruct, Left: ~const Bulk, Right: ~const Bulk>,
    N: Length<Elem = (), Value = NN> + ?Sized,
    F: ~const Fn() -> T::Item + ~const Destruct + Copy,
    NN: LengthValue<Metadata = N::Metadata, Length<()> = N, SaturatingSub<M> = R>,
    M: LengthValue,
    R: LengthValue
{
    type Left = ResizeWith<T::Left, F, N>;
    type Right = ResizeWith<T::Right, F, R::Length<()>>;

    fn split_at(Self { bulk, n, padder }: Self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let n = NN::from_metadata(n);
        let (left, right) = bulk.split_at(m);
        (
            left.resize_with(n, padder),
            right.resize_with(length::value::saturating_sub(n, m), padder)
        )
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::range([(); 0], [(); 7])
            .rev()
            .resize_with([(); 10], &|| 7)
            .collect::<Vec<_>, _>();

        println!("{a:?}")
    }
}