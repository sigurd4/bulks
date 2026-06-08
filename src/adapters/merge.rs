use array_trait::length::{self, LengthValue};

use crate::{Bulk, DoubleEndedBulk, IntoBulk, SplitBulk};

pub const fn merge<Lhs, Rhs, F>(lhs: impl ~const IntoBulk<IntoBulk = Lhs>, rhs: impl ~const IntoBulk<IntoBulk = Rhs>, f: F) -> Merge<Lhs, Rhs, F>
where
    Lhs: Bulk<Item: Into<<F as FnOnce<(Lhs::Item, Rhs::Item)>>::Output>>,
    Rhs: Bulk<Item: Into<<F as FnOnce<(Lhs::Item, Rhs::Item)>>::Output>>,
    F: FnMut<(Lhs::Item, Rhs::Item)>
{
    Merge::new(lhs.into_bulk(), rhs.into_bulk(), f)
}

pub struct Merge<Lhs, Rhs, F>
where
    Lhs: Bulk<Item: Into<<F as FnOnce<(Lhs::Item, Rhs::Item)>>::Output>>,
    Rhs: Bulk<Item: Into<<F as FnOnce<(Lhs::Item, Rhs::Item)>>::Output>>,
    F: FnMut<(Lhs::Item, Rhs::Item)>
{
    lhs: Lhs,
    rhs: Rhs,
    f: F
}

impl<Lhs, Rhs, F, O> Merge<Lhs, Rhs, F>
where
    Lhs: Bulk<Item: Into<O>>,
    Rhs: Bulk<Item: Into<O>>,
    F: FnMut(Lhs::Item, Rhs::Item) -> O
{
    pub(crate) const fn new(lhs: Lhs, rhs: Rhs, f: F) -> Self
    {
        Self {
            lhs,
            rhs,
            f
        }
    }
}

const fn merge_once<F, Lhs, Rhs, O>(f: &mut F, lhs: Option<Lhs>, rhs: Option<Rhs>) -> Option<O>
where
    F: ~const FnMut(Lhs, Rhs) -> O,
    Lhs: ~const Into<O>,
    Rhs: ~const Into<O>
{
    match (lhs, rhs)
    {
        (Some(lhs), Some(rhs)) => Some(f(lhs, rhs)),
        (None, Some(x)) => Some(x.into()),
        (Some(x), None) => Some(x.into()),
        (None, None) => None
    }
}

mod private
{
    use crate::{adapters::merge::merge_once};

    pub struct IntoIter<Lhs, Rhs, F, O>
    where
        Lhs: Iterator<Item: Into<O>>,
        Rhs: Iterator<Item: Into<O>>,
        F: FnMut(Lhs::Item, Rhs::Item) -> O
    {
        pub lhs: Lhs,
        pub rhs: Rhs,
        pub f: F
    }

    impl<Rhs, Lhs, F, O> Iterator for IntoIter<Lhs, Rhs, F, O>
    where
        Lhs: Iterator<Item: Into<O>>,
        Rhs: Iterator<Item: Into<O>>,
        F: FnMut(Lhs::Item, Rhs::Item) -> O
    {
        type Item = O;
        
        fn next(&mut self) -> Option<Self::Item>
        {
            let Self { lhs, rhs, f } = self;
            merge_once(f, lhs.next(), rhs.next())
        }
        fn size_hint(&self) -> (usize, Option<usize>)
        {
            let Self { lhs, rhs, f: _ } = self;
            let (lhs_min, lhs_max) = lhs.size_hint();
            let (rhs_min, rhs_max) = rhs.size_hint();
            (
                lhs_min.max(rhs_min),
                merge_once(&mut |lhs_max: usize, rhs_max: usize| lhs_max.max(rhs_max), lhs_max, rhs_max)
            )
        }
    }
    impl<Rhs, Lhs, F, O> ExactSizeIterator for IntoIter<Lhs, Rhs, F, O>
    where
        Lhs: ExactSizeIterator<Item: Into<O>>,
        Rhs: ExactSizeIterator<Item: Into<O>>,
        F: FnMut(Lhs::Item, Rhs::Item) -> O
    {
        fn is_empty(&self) -> bool
        {
            let Self { lhs, rhs, f: _ } = self;
            lhs.is_empty() && rhs.is_empty()
        }
        fn len(&self) -> usize
        {
            let Self { lhs, rhs, f: _ } = self;
            lhs.len().max(rhs.len())
        }
    }
    impl<Rhs, Lhs, F, O> DoubleEndedIterator for IntoIter<Lhs, Rhs, F, O>
    where
        Lhs: DoubleEndedIterator<Item: Into<O>> + ExactSizeIterator,
        Rhs: DoubleEndedIterator<Item: Into<O>> + ExactSizeIterator,
        F: FnMut(Lhs::Item, Rhs::Item) -> O
    {
        fn next_back(&mut self) -> Option<Self::Item>
        {
            let Self { lhs, rhs, f } = self;
            let lhs_len = lhs.len();
            let rhs_len = rhs.len();
            if lhs_len > rhs_len
            {
                return lhs.next_back().map(Into::into)
            }
            if rhs_len > lhs_len
            {
                return rhs.next_back().map(Into::into)
            }
            merge_once(f, lhs.next_back(), rhs.next_back())
        }
    }
}

impl<Lhs, Rhs, F, O> IntoIterator for Merge<Lhs, Rhs, F>
where
    Lhs: Bulk<Item: Into<O>>,
    Rhs: Bulk<Item: Into<O>>,
    F: FnMut(Lhs::Item, Rhs::Item) -> O
{
    type Item = O;
    type IntoIter = private::IntoIter<Lhs::IntoIter, Rhs::IntoIter, F, O>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { lhs, rhs, f } = self;
        private::IntoIter {
            lhs: lhs.into_iter(),
            rhs: rhs.into_iter(),
            f
        }
    }
}
impl<Lhs, Rhs, F, O> Bulk for Merge<Lhs, Rhs, F>
where
    Lhs: Bulk<Item: Into<O>>,
    Rhs: Bulk<Item: Into<O>>,
    F: FnMut(Lhs::Item, Rhs::Item) -> O
{
    type Length = length::Max<Lhs::Length, Rhs::Length>;
    type MaxLength = length::Max<Lhs::MaxLength, Rhs::MaxLength>;
    type MinLength = length::Max<Lhs::MinLength, Rhs::MinLength>;

    fn len(&self) -> usize
    {
        let Self { lhs, rhs, f: _ } = self;
        Ord::max(lhs.len(), rhs.len())
    }
    fn is_empty(&self) -> bool
    {
        let Self { lhs, rhs, f: _ } = self;
        lhs.is_empty() && rhs.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        let Self { lhs, rhs, mut f } = self;
        merge_once(&mut f, lhs.first(), rhs.first())
    }

    fn for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: FnMut(Self::Item)
    {
        self.into_iter().for_each(f)
    }
    fn try_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        FF: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        self.into_iter().try_for_each(f)
    }
}
impl<Lhs, Rhs, F, O> DoubleEndedBulk for Merge<Lhs, Rhs, F>
where
    Lhs: DoubleEndedBulk<Item: Into<O>>,
    Rhs: DoubleEndedBulk<Item: Into<O>>,
    F: FnMut(Lhs::Item, Rhs::Item) -> O
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: FnMut(Self::Item)
    {
        self.into_iter().rev().for_each(f)
    }
    fn try_rev_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        FF: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        self.into_iter().rev().try_for_each(f)
    }
}
impl<Lhs, Rhs, F, O, L> const SplitBulk<L> for Merge<Lhs, Rhs, F>
where
    Lhs: ~const SplitBulk<L, Item: Into<O>>,
    Rhs: ~const SplitBulk<L, Item: Into<O>>,
    Self: ~const Bulk,
    Merge<Lhs::Left, Rhs::Left, F>: ~const Bulk<Item = Self::Item>,
    Merge<Lhs::Right, Rhs::Right, F>: ~const Bulk<Item = Self::Item>,
    F: FnMut(Lhs::Item, Rhs::Item) -> O + Copy,
    L: LengthValue
{
    type Left = Merge<Lhs::Left, Rhs::Left, F>;
    type Right = Merge<Lhs::Right, Rhs::Right, F>;

    fn split_at(Self { lhs, rhs, f }: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let (a_left, a_right) = lhs.split_at(n);
        let (b_left, b_right) = rhs.split_at(n);
        
        (
            Merge::new(a_left, b_left, f),
            Merge::new(a_right, b_right, f),
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
        let a = [1, 3, 5];
        let b = [2, 4, 6, 7];
        let bulk = a.into_bulk()
            .merge(b, |a, b| a + b);
        let c = bulk.collect::<[_; _], _>();
        
        println!("{c:?}");
        assert_eq!(c, [3, 7, 11, 7])
    }
}