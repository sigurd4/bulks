use core::{marker::Destruct, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};

use crate::{Bulk, DoubleEndedBulk, RepeatNWith, SplitBulk, util::YieldOnce};

/// Creates a new bulk that repeats a single element a given number of times.
///
/// The `repeat_n()` function repeats a single value exactly `n` times.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// // four of the number four:
/// let four_fours: [_; _] = bulks::repeat_n(4, [(); 4]).collect();
/// 
/// assert_eq!(four_fours, [4, 4, 4, 4]);
/// ```
///
/// For non-`Copy` types,
///
/// ```
/// # #![feature(generic_const_exprs)]
/// use bulks::*;
///
/// let v: Vec<i32> = Vec::with_capacity(123);
/// let mut bulk = bulks::repeat_n(v, [(); 5]);
/// 
/// let (first_four, last) = bulk.split_at([(); 4]);
///
/// for cloned in first_four
/// {
///     // It starts by cloning things
///     assert_eq!(cloned.len(), 0);
///     assert_eq!(cloned.capacity(), 0);
/// }
///
/// // ... but the last item is the original one
/// let [last] = last.collect();
/// assert_eq!(last.len(), 0);
/// assert_eq!(last.capacity(), 123);
/// ```
pub const fn repeat_n<T, L>(element: T, n: L) -> RepeatN<T, L::Length<()>>
where
    T: Clone,
    L: LengthValue
{
    RepeatN::new(element, n)
}

/// A bulk that repeats an element an exact number of times.
///
/// This `struct` is created by the [`repeat_n()`] function.
/// See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct RepeatN<A, N = [()]>
where
    A: Clone,
    N: Length<Elem = ()> + ?Sized
{
    element: A,
    n: <N as Pointee>::Metadata
}

impl<A, N> Clone for RepeatN<A, N>
where
    A: Clone,
    N: Length<Elem = ()> + ?Sized
{
    fn clone(&self) -> Self
    {
        let Self { element, n } = self;
        Self {
            element: element.clone(),
            n: *n
        }
    }
}

impl<A, N> RepeatN<A, N>
where
    A: Clone,
    N: Length<Elem = ()> + ?Sized
{
    const fn new(element: A, n: length::Value<N>) -> RepeatN<A, N>
    {
        RepeatN {
            element,
            n: length::value::into_metadata(n)
        }
    }
}

impl<A, N> core::fmt::Debug for RepeatN<A, N>
where
    A: Clone + core::fmt::Debug,
    N: Length<Elem = ()> + ?Sized
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        f.debug_struct("RepeatN").field("count", &self.len()).field("element", &self.element).finish()
    }
}

/*const*/ impl<A, N> IntoIterator for RepeatN<A, N>
where
    A: Clone,
    N: Length<Elem = ()> + ?Sized
{
    type Item = A;
    type IntoIter = core::iter::RepeatN<A>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { element, n } = self;
        core::iter::repeat_n(element, length::len_metadata::<N>(n))
    }
}
const impl<A, N> Bulk for RepeatN<A, N>
where
    A: ~const Clone + ~const Destruct,
    N: Length<Elem = ()> + ?Sized
{
    type MinLength = N;
    type MaxLength = N;

    fn len(&self) -> usize
    {
        let Self { element: _, n } = self;
        length::len_metadata::<N>(*n)
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { element, n } = self;
        let n = length::len_metadata::<N>(n);
        let mut i = 1;
        while i < n
        {
            f(element.clone());
            i += 1
        }
        if i == n
        {
            f(element)
        }
    }
    fn try_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { element, n } = self;
        let n = length::len_metadata::<N>(n);
        let mut i = 1;
        while i < n
        {
            f(element.clone())?;
            i += 1
        }
        if i == n
        {
            f(element)?
        }
        R::from_output(())
    }
}
const impl<A, N> DoubleEndedBulk for RepeatN<A, N>
where
    A: ~const Clone + ~const Destruct,
    N: Length<Elem = ()> + ?Sized
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        self.for_each(f);
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        self.try_for_each(f)
    }
}
const impl<A, N, M, L, R> SplitBulk<M> for RepeatN<A, N>
where
    N: Length<Elem = (), Value: LengthValue<Min<M> = L, SaturatingSub<M> = R>>,
    A: ~const Clone + ~const Destruct,
    M: LengthValue,
    L: LengthValue,
    R: LengthValue
{
    type Left = RepeatN<A, L::Length<()>>;
    type Right = RepeatN<A, R::Length<()>>;

    fn split_at(Self { element, n }: Self, m: M) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let n = length::value::from_metadata::<N::Value>(n);
        (
            RepeatN::new(element.clone(), length::value::min(n, m)),
            RepeatN::new(element, length::value::saturating_sub(n, m))
        )
    }
}

const impl<A, N> From<RepeatN<A, N>> for RepeatNWith<YieldOnce<A>, N>
where
    A: Clone,
    N: Length<Elem = ()>
{
    fn from(value: RepeatN<A, N>) -> Self
    {
        let RepeatN { element, n } = value;
        crate::repeat_n_with(YieldOnce::new(element), length::value::from_metadata::<N::Value>(n))
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::repeat_n(1, [(); 4])
            .collect::<[_; _], _>();
        assert_eq!(a, [1, 1, 1, 1])
    }

    #[test]
    fn doctest()
    {
        use crate::*;

        let v: Vec<i32> = Vec::with_capacity(123);
        let bulk = crate::repeat_n(v, [(); 5]);

        let (first_four, last) = bulk.split_at([(); 4]);

        for cloned in first_four
        {
            // It starts by cloning things
            assert_eq!(cloned.len(), 0);
            assert_eq!(cloned.capacity(), 0);
        }

        // ... but the last item is the original one
        let [last] = last.collect();
        assert_eq!(last.len(), 0);
        assert_eq!(last.capacity(), 123);
    }
}