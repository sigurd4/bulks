use core::{marker::Destruct, ops::Try};

use array_trait::length::{self, Length, LengthValue};

use crate::{Bulk, DoubleEndedBulk, SplitBulk, range::Range};

#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct RangeInclusive<S, E = S>
where
    S: Length<Elem = ()> + ?Sized,
    E: Length<Elem = ()> + ?Sized
{
    start: S::Metadata,
    end: E::Metadata
}

/*impl IntoBulk for core::ops::RangeInclusive<usize>
{
    type IntoBulk = RangeInclusive<[()]>;

    fn into_bulk(self) -> Self::IntoBulk
    {
        crate::range_inclusive(*self.start(), *self.end())
    }
}*/
impl<S, E> RangeInclusive<S, E>
where
    S: Length<Elem = ()> + ?Sized,
    E: Length<Elem = ()> + ?Sized
{
    pub(crate) const fn new(start: S::Value, end: E::Value) -> Self
    {
        Self {
            start: length::value::into_metadata(start),
            end: length::value::into_metadata(end)
        }
    }

    pub const fn start(&self) -> S::Value
    {
        length::value::from_metadata::<S::Value>(self.start)
    }
    pub const fn end(&self) -> E::Value
    {
        length::value::from_metadata::<E::Value>(self.end)
    }

    const fn in_range<N>(&self, n: N) -> Option<usize>
    where
        N: LengthValue
    {
        if length::value::ge(n, self.start())
            && length::value::le(n, self.end())
        {
            return Some(length::value::len(n))
        }
        None
    }
}

impl<S, E> IntoIterator for RangeInclusive<S, E>
where
    S: Length<Elem = ()> + ?Sized,
    E: Length<Elem = ()> + ?Sized
{
    type IntoIter = core::ops::Range<usize>;
    type Item = usize;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { start, end } = self;
        length::len_metadata::<S>(start)..(length::len_metadata::<E>(end) + 1)
    }
}
impl<S, E> const Bulk for RangeInclusive<S, E>
where
    S: Length<Elem = ()> + ?Sized,
    E: Length<Elem = ()> + ?Sized
{
    type Length = length::SaturatingSub<E, S>;
    type MinLength = length::SaturatingSub<E, S>;
    type MaxLength = length::SaturatingSub<E, S>;
    
    fn len(&self) -> usize
    {
        let Self { start, end } = self;
        (length::len_metadata::<E>(*end) + 1).saturating_sub(length::len_metadata::<S>(*start))
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        self.in_range(self.start())
    }
    fn last(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        self.in_range(self.end())
    }
    fn nth<L>(self, n: L) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        L: LengthValue
    {
        self.in_range(length::value::add(self.start(), n))
    }

    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { start, end } = self;
        let mut range = length::len_metadata::<S>(start)..length::len_metadata::<E>(end);
        while range.start < range.end
        {
            f(range.start);
            range.start += 1
        }
        if range.start == range.end
        {
            f(range.start)
        }
    }
    fn try_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { start, end } = self;
        let mut range = length::len_metadata::<S>(start)..length::len_metadata::<E>(end);
        while range.start < range.end
        {
            f(range.start)?;
            range.start += 1
        }
        if range.start == range.end
        {
            f(range.start)?
        }
        R::from_output(())
    }
}
impl<S, E> const DoubleEndedBulk for RangeInclusive<S, E>
where
    S: Length<Elem = ()> + ?Sized,
    E: Length<Elem = ()> + ?Sized
{
    fn rev_for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { start, end } = self;
        let mut range = length::len_metadata::<S>(start)..length::len_metadata::<E>(end);
        while range.start < range.end
        {
            f(range.end);
            range.end -= 1
        }
        if range.start == range.end
        {
            f(range.end)
        }
    }

    fn try_rev_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { start, end } = self;
        let mut range = length::len_metadata::<S>(start)..length::len_metadata::<E>(end);
        while range.start < range.end
        {
            f(range.end)?;
            range.end -= 1
        }
        if range.start == range.end
        {
            f(range.end)?
        }
        R::from_output(())
    }
}
impl<S, E, L> const SplitBulk<L> for RangeInclusive<S, E>
where
    S: Length<Elem = ()> + ?Sized,
    E: Length<Elem = ()> + ?Sized,
    L: LengthValue
{
    type Left = Range<S, length::value::Length<length::value::Min<E::Value, length::value::Add<S::Value, L>>, ()>>;
    type Right = RangeInclusive<length::value::Length<length::value::Min<E::Value, length::value::Add<S::Value, L>>, ()>, E>;

    fn split_at(Self { start, end }: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let mid = length::value::into_metadata(
            length::value::min(length::value::from_metadata::<E::Value>(end), length::value::add(length::value::from_metadata::<S::Value>(start), n))
        );
        (
            Range { start, end: mid },
            RangeInclusive { start: mid, end }
        )
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        for (i, a) in crate::range_inclusive([(); 1], [(); 8])
            .enumerate()
            .rev()
        {
            assert!(a <= 8);
            assert_eq!(i + 1, a)
        }
    }
}