use core::{range::Step, ops::Try};

use crate::{Bulk, DoubleEndedBulk, SplitBulk, StaticBulk, util::{LengthSpec, Stepper}};

/// A bulk that yields the element's index counting from a given initial index and the element.
///
/// This `struct` is created by the [`enumerate`](Bulk::enumerate) method on [`Bulk`]. See its
/// documentation for more.
#[derive(Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct EnumerateFrom<I, U>
where
    I: Bulk,
    U: Step + Copy
{
    bulk: I,
    initial_count: U
}

impl<I, T, U> EnumerateFrom<I, U>
where
    I: Bulk<Item = T>,
    U: Step + Copy
{
    pub(crate) const fn new(bulk: I, initial_count: U) -> Self
    {
        Self {
            bulk,
            initial_count
        }
    }
}

impl<I, T, U> IntoIterator for EnumerateFrom<I, U>
where
    I: Bulk<Item = T>,
    U: Step + Copy
{
    type IntoIter = core::iter::Map<I::IntoIter, Stepper<U>>;
    type Item = (U, T);

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, initial_count } = self;
        bulk.into_iter()
            .map(Stepper::new(initial_count))
    }
}
impl<I, T, U> Bulk for EnumerateFrom<I, U>
where
    I: Bulk<Item = T>,
    U: Step + Copy
{
    fn len(&self) -> usize
    {
        let Self { bulk, initial_count: _ } = self;
        bulk.len()
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, initial_count: _ } = self;
        bulk.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        let Self { bulk, initial_count } = self;
        bulk.first().map(Stepper::<_, false>::new(initial_count))
    }
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        let Self { bulk, initial_count } = self;
        let len = bulk.len();
        bulk.last()
            .map(|last| (U::forward(initial_count, len - 1), last))
    }
    fn nth<L>(self, n: L) -> Option<Self::Item>
    where
        Self: Sized,
        L: LengthSpec
    {
        let Self { bulk, initial_count } = self;
        bulk.nth(n)
            .map(|nth| (U::forward(initial_count, n.len_metadata()), nth))
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        let Self { bulk, initial_count } = self;
        bulk.for_each(Closure {
            i: Stepper::<_, false>::new(initial_count),
            f
        })
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        let Self { bulk, initial_count } = self;
        bulk.try_for_each(Closure {
            i: Stepper::<_, false>::new(initial_count),
            f
        })
    }
}
impl<I, T, U> DoubleEndedBulk for EnumerateFrom<I, U>
where
    I: DoubleEndedBulk<Item = T> + Bulk,
    U: Step + Copy
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        let Self { bulk, initial_count } = self;
        let i = bulk.len();
        bulk.rev_for_each(Closure {
            i: Stepper::<_, true>::new(U::forward(initial_count, i.saturating_sub(1))),
            f
        })
    }

    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        let Self { bulk, initial_count } = self;
        let i = bulk.len();
        bulk.try_rev_for_each(Closure {
            i: Stepper::<_, true>::new(U::forward(initial_count, i.saturating_sub(1))),
            f
        })
    }
}
unsafe impl<I, T, U, const N: usize> StaticBulk for EnumerateFrom<I, U>
where 
    I: StaticBulk<Item = T, Array<T> = [T; N]>,
    U: Step + Copy
{
    type Array<V> = [V; N];
}
impl<I, T, U, L> SplitBulk<L> for EnumerateFrom<I, U>
where
    I: SplitBulk<L, Item = T>,
    U: Step + Copy,
    L: LengthSpec
{
    type Left = EnumerateFrom<I::Left, U>;
    type Right = EnumerateFrom<I::Right, U>;

    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { bulk, initial_count } = self;
        let (left, right) = bulk.split_at(n);
        let following_count = U::forward(initial_count, left.len());
        (
            left.enumerate_from(initial_count),
            right.enumerate_from(following_count)
        )
    }
}

struct Closure<F, U, const REV: bool>
where
    U: Step + Copy
{
    i: Stepper<U, REV>,
    f: F
}
impl<F, T, U, R, const REV: bool> const FnOnce<(T,)> for Closure<F, U, REV>
where
    F: ~const FnOnce((U, T)) -> R,
    U: Step + Copy,
    Stepper<U, REV>: ~const FnOnce(T) -> (U, T)
{
    type Output = R;

    extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output
    {
        let Self { i, f } = self;
        f(i.call_once(args))
    }
}
impl<F, T, U, R, const REV: bool> const FnMut<(T,)> for Closure<F, U, REV>
where
    F: ~const FnMut((U, T)) -> R,
    U: Step + Copy,
    Stepper<U, REV>: ~const FnMut(T) -> (U, T)
{
    extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output
    {
        let Self { i, f } = self;
        f(i.call_mut(args))
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = ['1', '2', '3', '4', '5', '6', '7', '8'];
        
        for (i, a) in a.into_bulk()
            .enumerate_from(1)
        {
            assert_eq!(i, a.to_string().parse().unwrap())
        }
    }

    #[test]
    fn zipped()
    {
        let enumerate: [_; _] = (*b"foo").into_bulk().enumerate_from(1).collect();
        
        let zipper: Vec<_> = crate::rzip(1.., *b"foo").collect();
        
        assert_eq!((1, b'f'), enumerate[0]);
        assert_eq!((1, b'f'), zipper[0]);
        
        assert_eq!((2, b'o'), enumerate[1]);
        assert_eq!((2, b'o'), zipper[1]);
        
        assert_eq!((3, b'o'), enumerate[2]);
        assert_eq!((3, b'o'), zipper[2]);
    }
}