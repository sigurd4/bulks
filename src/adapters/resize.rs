use core::{marker::Destruct, ops::Try, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};

use crate::{Bulk, DoubleEndedBulk, IntoBulk, IntoContained, SplitBulk};

pub const fn resize<I, L>(iterable: I, n: L, element: I::Item) -> Resize<
    <<I as IntoContained>::IntoContained as IntoBulk>::IntoBulk,
    L::Length<()>
>
where
    I: ~const IntoContained<Item: Copy>,
    L: LengthValue
{
    unsafe {
        Resize::new(iterable.into_contained().into_bulk(), n, element)
    }
}

/// A bulk that only delivers exactly `n` elements, taking the first at most `n` elements of `bulk`, then produces copies of `element`.
///
/// This `struct` is created by the [`resize`](Bulk::resize) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Resize<T, N = [()]>
where
    T: Bulk<Item: Copy>,
    N: Length<Elem = ()> + ?Sized
{
    bulk: T,
    n: <N as Pointee>::Metadata,
    element: T::Item
}

impl<T, N> Resize<T, N>
where
    T: Bulk<Item: Copy>,
    N: Length<Elem = ()> + ?Sized
{
    pub(crate) const fn new(bulk: T, n: N::Value, element: T::Item) -> Resize<T, N>
    {
        Self { bulk, n: length::value::into_metadata(n), element }
    }
}
/*const*/ impl<T, N> IntoIterator for Resize<T, N>
where
    T: Bulk<Item: Copy>,
    N: Length<Elem = ()> + ?Sized
{
    type Item = T::Item;
    type IntoIter = <<core::iter::Take<core::iter::Chain<T::IntoIter, core::iter::Repeat<T::Item>>> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, n, element } = self;
        unsafe {
            bulk.into_iter()
                .chain(core::iter::repeat(element))
                .take(length::len_metadata::<N>(n))
                .into_contained()
                .into_iter()
        }
    }
}
const impl<T, N> Bulk for Resize<T, N>
where
    T: ~const Bulk<Item: Copy + ~const Destruct>,
    N: Length<Elem = ()> + ?Sized
{
    type MinLength = N;
    type MaxLength = N;

    fn len(&self) -> usize
    {
        let Self { bulk: _, n, element: _ } = self;
        length::len_metadata::<N>(*n)
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, n, element } = self;
        let mut m = bulk.len();
        bulk.take(length::value::from_metadata::<N::Value>(n))
            .for_each(&mut f);
        while m < length::len_metadata::<N>(n)
        {
            f(element);
            m += 1
        }
    }
    fn try_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, n, element } = self;
        let mut m = bulk.len();
        bulk.take(length::value::from_metadata::<N::Value>(n))
            .try_for_each(&mut f)?;
        while m < length::len_metadata::<N>(n)
        {
            f(element)?;
            m += 1
        }
        R::from_output(())
    }
}
const impl<T, N> DoubleEndedBulk for Resize<T, N>
where
    T: ~const DoubleEndedBulk<Item: Copy + ~const Destruct> + ~const Bulk + ~const Destruct,
    N: Length<Elem = ()> + ?Sized,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk, n, element } = self;
        let mut m = bulk.len();
        while m < length::len_metadata::<N>(n)
        {
            f(element);
            m += 1
        }
        let m = bulk.length();
        bulk.rev()
            .skip(length::value::saturating_sub(m, length::value::from_metadata::<N::Value>(n)))
            .take(length::value::from_metadata::<N::Value>(n))
            .for_each(f);
    }
    fn try_rev_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk, n, element } = self;
        let mut m = bulk.len();
        while m < length::len_metadata::<N>(n)
        {
            f(element)?;
            m += 1
        }
        let m = bulk.length();
        bulk.rev()
            .skip(length::value::saturating_sub(m, length::value::from_metadata::<N::Value>(n)))
            .take(length::value::from_metadata::<N::Value>(n))
            .try_for_each(f)
    }
}
const impl<T, N, NN, M, R> SplitBulk<M> for Resize<T, N>
where
    T: ~const SplitBulk<M, Item: Copy + ~const Destruct, Left: ~const Bulk, Right: ~const Bulk>,
    N: Length<Elem = (), Value = NN> + ?Sized,
    NN: LengthValue<Metadata = N::Metadata, Length<()> = N, SaturatingSub<M> = R>,
    M: LengthValue,
    R: LengthValue
{
    type Left = Resize<T::Left, N>;
    type Right = Resize<T::Right, R::Length<()>>;

    fn split_at(Self { bulk, n, element }: Self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let n = NN::from_metadata(n);
        let (left, right) = bulk.split_at(m);
        (
            left.resize(n, element),
            right.resize(length::value::saturating_sub(n, m), element)
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
        let a = crate::resize(0..=6, [(); 10], 7).collect::<Vec<_>, _>();

        println!("{a:?}")
    }
}