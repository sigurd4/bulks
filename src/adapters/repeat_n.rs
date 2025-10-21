use core::{fmt, marker::Destruct, ptr::Pointee};

use crate::{util::{Length, LengthSpec}, Bulk, DoubleEndedBulk, StaticBulk};

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
/// let four_fours = bulks::repeat_n(4, 4)
///     .collect_array();
/// 
/// assert_eq!(four_fours, [4, 4, 4, 4]);
/// ```
///
/// For non-`Copy` types,
///
/// ```
/// use std::iter;
///
/// let v: Vec<i32> = Vec::with_capacity(123);
/// let mut reps = iter::repeat_n(v, 5).collect();
///
/// for cloned in &reps[0..4] {
///     // It starts by cloning things
///     assert_eq!(cloned.len(), 0);
///     assert_eq!(cloned.capacity(), 0);
/// }
///
/// // ... but the last item is the original one
/// let last = it.last().unwrap();
/// assert_eq!(last.len(), 0);
/// assert_eq!(last.capacity(), 123);
/// ```
#[allow(invalid_type_param_default)]
pub const fn repeat_n<T, L>(element: T, n: L) -> RepeatN<T, L::Length<T>>
where
    T: Clone,
    L: ~const LengthSpec
{
    RepeatN {
        element,
        n: n.into_metadata()
    }
}

/// A bulk that repeats an element an exact number of times.
///
/// This `struct` is created by the [`repeat_n()`] function.
/// See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct RepeatN<A, N = [A]>
where
    A: Clone,
    N: Length<Elem = A> + ?Sized
{
    element: A,
    n: <N as Pointee>::Metadata
}

impl<A, N> fmt::Debug for RepeatN<A, N>
where
    A: Clone + fmt::Debug,
    N: Length<Elem = A> + ?Sized
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("RepeatN").field("count", &self.len()).field("element", &self.element).finish()
    }
}

impl<A, N> IntoIterator for RepeatN<A, N>
where
    A: Clone,
    N: Length<Elem = A> + ?Sized
{
    type Item = A;
    type IntoIter = core::iter::RepeatN<A>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { element, n } = self;
        core::iter::repeat_n(element, N::len_metadata(n))
    }
}
impl<A, N> const Bulk for RepeatN<A, N>
where
    A: ~const Clone + ~const Destruct,
    N: ~const Length<Elem = A> + ?Sized
{
    fn len(&self) -> usize
    {
        let Self { element: _, n } = self;
        N::len_metadata(*n)
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { element, n } = self;
        let n = N::len_metadata(n);
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
        let n = N::len_metadata(n);
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
impl<A, N> const DoubleEndedBulk for RepeatN<A, N>
where
    A: ~const Clone + ~const Destruct,
    N: ~const Length<Elem = A> + ?Sized
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
unsafe impl<A, const N: usize> StaticBulk for RepeatN<A, [A; N]>
where
    A: Clone
{
    type Array<U> = [U; N];
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::repeat_n(1, [(); _]).collect::<[_; _]>();
        assert_eq!(a, [1, 1, 1, 1])
    }
}