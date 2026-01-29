use core::{borrow::Borrow, fmt, marker::{Destruct, PhantomData}, ptr::Pointee};

use array_trait::length::{self, Length, LengthValue};
use currying::{Curry, RCurry};

use crate::{Bulk, DoubleEndedBulk, RandomAccessBulk, RandomAccessBulkSpec, RepeatNWith, SplitBulk, util::YieldOnce};

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
#[derive(Clone)]
pub struct RepeatN<A, N = [()], P = A>
where
    A: Clone + Borrow<P>,
    N: Length<Elem = ()> + ?Sized
{
    element: A,
    n: <N as Pointee>::Metadata,
    marker: PhantomData<P>
}

impl<A, N, P> RepeatN<A, N, P>
where
    A: Clone + Borrow<P>,
    N: Length<Elem = ()> + ?Sized
{
    const fn new(element: A, n: length::Value<N>) -> RepeatN<A, N, P>
    {
        RepeatN {
            element,
            n: length::value::into_metadata(n),
            marker: PhantomData
        }
    }
}

impl<A, N, P> fmt::Debug for RepeatN<A, N, P>
where
    A: Clone + fmt::Debug + Borrow<P>,
    N: Length<Elem = ()> + ?Sized
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("RepeatN").field("count", &self.len()).field("element", &self.element).finish()
    }
}

impl<A, N, P> IntoIterator for RepeatN<A, N, P>
where
    A: Clone + Borrow<P>,
    N: Length<Elem = ()> + ?Sized
{
    type Item = A;
    type IntoIter = core::iter::RepeatN<A>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { element, n, marker: PhantomData } = self;
        core::iter::repeat_n(element, length::len_metadata::<N>(n))
    }
}
impl<A, N, P> const Bulk for RepeatN<A, N, P>
where
    A: ~const Clone + ~const Destruct + Borrow<P>,
    N: Length<Elem = ()> + ?Sized
{
    type MinLength = N;
    type MaxLength = N;

    fn len(&self) -> usize
    {
        let Self { element: _, n, marker: PhantomData } = self;
        length::len_metadata::<N>(*n)
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { element, n, marker: PhantomData } = self;
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
        let Self { element, n, marker: PhantomData } = self;
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
impl<A, N, P> const DoubleEndedBulk for RepeatN<A, N, P>
where
    A: ~const Clone + ~const Destruct + Borrow<P>,
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
impl<A, N, M, L, R, P> const SplitBulk<M> for RepeatN<A, N, P>
where
    N: Length<Elem = (), Value: LengthValue<Min<M> = L, SaturatingSub<M> = R>>,
    A: ~const Clone + ~const Destruct + Borrow<P>,
    M: LengthValue,
    L: LengthValue,
    R: LengthValue
{
    type Left = RepeatN<A, L::Length<()>, P>;
    type Right = RepeatN<A, R::Length<()>, P>;

    fn split_at(Self { element, n, marker: PhantomData }: Self, m: M) -> (Self::Left, Self::Right)
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

impl<A, N, P> const RandomAccessBulk for RepeatN<A, N, P>
where
    A: ~const Clone + ~const Destruct + ~const Borrow<P>,
    N: Length<Elem = (), Metadata: ~const Destruct> + ?Sized
{
    type ItemPointee = P;
    type EachRef<'a> = RepeatN<&'a P, N, P>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(Self { element, n, marker: PhantomData }: &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        RepeatN::new(element.borrow(), length::value::from_metadata(*n))
    }
}
impl<A, N, P> const RandomAccessBulkSpec for RepeatN<A, N, P>
where
    A: ~const Clone + ~const Destruct + ~const Borrow<P>,
    N: Length<Elem = (), Metadata: ~const Destruct> + ?Sized
{
    fn _get<'a, L>(Self { element, n, marker: PhantomData }: &'a Self, i: L) -> Option<&'a <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: 'a
    {
        if length::value::ge(i, length::value::from_metadata::<N::Value>(*n))
        {
            return None
        }
        Some(element.borrow())
    }

    fn _get_many<'a, NN, const M: usize>(Self { element, n, marker: PhantomData }: &'a Self, i: NN) -> [Option<&'a <Self as RandomAccessBulk>::ItemPointee>; M]
    where
        Self: 'a,
        NN: ~const crate::IntoBulk<Item = usize, IntoBulk: ~const Bulk + crate::StaticBulk<Array<()> = [(); M]>>
    {
        const fn getidx<A, P, N>(element: &A, i: usize, n: N) -> Option<&P>
        where
            A: ~const Borrow<P>,
            N: LengthValue
        {
            if length::value::ge(i, n)
            {
                return None
            }
            Some(element.borrow())
        }

        i.into_bulk()
            .map(getidx::<A, P, N::Value>
                .curry_once(element)
                .rcurry_once(length::value::from_metadata::<N::Value>(*n))
            ).collect_array()
    }
}

impl<A, N> const From<RepeatN<A, N>> for RepeatNWith<YieldOnce<A>, N>
where
    A: Clone,
    N: Length<Elem = ()>
{
    fn from(value: RepeatN<A, N>) -> Self
    {
        let RepeatN { element, n, marker: PhantomData } = value;
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
        let a = crate::repeat_n(1, [(); _])
            .each_ref()
            .copied()
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