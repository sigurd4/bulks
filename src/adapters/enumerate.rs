use core::{marker::Destruct, ops::Try};

use array_trait::length;

use crate::{Bulk, DoubleEndedBulk, EnumerateFrom, SplitBulk, StaticBulk};

/// A bulk that yields the element's index and the element.
///
/// This `struct` is created by the [`enumerate`](Bulk::enumerate) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Enumerate<I>
where
    I: Bulk
{
    bulk: I,
}

impl<I, T> Enumerate<I>
where
    I: Bulk<Item = T>
{
    pub(crate) const fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }
}

impl<I, T> const Default for Enumerate<I>
where
    I: ~const Bulk<Item = T> + ~const Default
{
    fn default() -> Self
    {
        I::default().enumerate()
    }
}

impl<I, T> IntoIterator for Enumerate<I>
where
    I: Bulk<Item = T>
{
    type IntoIter = core::iter::Enumerate<I::IntoIter>;
    type Item = (usize, T);

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk } = self;
        bulk.into_iter().enumerate()
    }
}
impl<I, T> const Bulk for Enumerate<I>
where
    I: ~const Bulk<Item = T>,
    T: ~const Destruct
{
    type MinLength<U> = I::MinLength<U>;
    type MaxLength<U> = I::MaxLength<U>;
    
    fn len(&self) -> usize
    {
        let Self { bulk } = self;
        bulk.len()
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk } = self;
        bulk.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        const fn enumerate<T>(x: T) -> (usize, T)
        {
            (0, x)
        }

        let Self { bulk } = self;
        bulk.first().map(enumerate)
    }
    fn last(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        let Self { bulk } = self;
        let len = bulk.len();
        match bulk.last()
        {
            Some(last) => Some((len - 1, last)),
            None => None
        }
    }
    fn nth<L>(self, n: L) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        L: length::LengthValue
    {
        let Self { bulk } = self;
        match bulk.nth(n)
        {
            Some(last) => Some((length::value::len(n), last)),
            None => None
        }
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk } = self;
        bulk.for_each(Closure::<_, false> {
            i: 0,
            f
        })
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk } = self;
        bulk.try_for_each(Closure::<_, false> {
            i: 0,
            f
        })
    }
}
impl<I, T> const DoubleEndedBulk for Enumerate<I>
where
    I: ~const DoubleEndedBulk<Item = T> + ~const Bulk,
    T: ~const Destruct
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { bulk } = self;
        let i = bulk.len();
        bulk.rev_for_each(Closure::<_, true> {
            i,
            f
        })
    }

    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        let Self { bulk } = self;
        let i = bulk.len();
        bulk.try_rev_for_each(Closure::<_, true> {
            i,
            f
        })
    }
}
unsafe impl<I, T, const N: usize> StaticBulk for Enumerate<I>
where 
    I: StaticBulk<Item = T, Array<(usize, T)> = [(usize, T); N], MinLength<(usize, T)> = [(usize, T); N], MaxLength<(usize, T)> = [(usize, T); N]>
{
    type Array<U> = [U; N];
}
impl<I, T, L> const SplitBulk<L> for Enumerate<I>
where
    I: ~const SplitBulk<L, Item = T, Left: ~const Bulk, Right: ~const Bulk>,
    L: length::LengthValue
{
    type Left = Enumerate<I::Left>;
    type Right = EnumerateFrom<I::Right, usize>;

    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { bulk } = self;
        let (left, right) = bulk.split_at(n);
        let following_count = left.len();
        (
            left.enumerate(),
            right.enumerate_from(following_count)
        )
    }
}

struct Closure<F, const REV: bool>
{
    i: usize,
    f: F
}
impl<F, T, R, const REV: bool> const FnOnce<(T,)> for Closure<F, REV>
where
    F: ~const FnOnce((usize, T)) -> R
{
    type Output = R;

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        (self.f)((self.i - REV as usize, x))
    }
}
impl<F, T, R, const REV: bool> const FnMut<(T,)> for Closure<F, REV>
where
    F: ~const FnMut((usize, T)) -> R
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        if REV
        {
            self.i -= 1;
        }
        let result = (self.f)((self.i, x));
        if !REV
        {
            self.i += 1;
        }
        result
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = ['0', '1', '2', '3', '4', '5', '6', '7'];
        
        for (i, a) in a.into_bulk()
            .enumerate()
        {
            assert_eq!(i, a.to_string().parse().unwrap())
        }
    }

    #[test]
    fn zipped()
    {
        let enumerate: [_; _] = (*b"foo").into_bulk().enumerate().collect();
        
        let zipper: Vec<_> = crate::rzip(0.., *b"foo").collect();
        
        assert_eq!((0, b'f'), enumerate[0]);
        assert_eq!((0, b'f'), zipper[0]);
        
        assert_eq!((1, b'o'), enumerate[1]);
        assert_eq!((1, b'o'), zipper[1]);
        
        assert_eq!((2, b'o'), enumerate[2]);
        assert_eq!((2, b'o'), zipper[2]);
    }
}