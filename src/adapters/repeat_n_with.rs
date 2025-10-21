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

/// Creates a new bulk that repeats elements of type `A` a given number of times
/// applying the provided closure, the repeater, `F: FnMut() -> A`.
///
/// The `repeat_n_with()` function calls the repeater a set amount of times.
///
/// If the element type of the iterator you need implements [`Clone`], and
/// it is OK to keep the source element in memory, you should instead use
/// the [`repeat_n()`](crate::repeat_n) function.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// // let's assume we have some value of a type that is not `Clone`
/// // or which we don't want to have in memory just yet because it is expensive:
/// #[derive(PartialEq, Debug)]
/// struct Expensive;
///
/// // a particular value forever:
/// let mut things = bulks::repeat_n_with(|| Expensive, 4).collect::<[_; _]>();
///
/// assert_eq!(things, [Expensive, Expensive, Expensive, Expensive])
/// ```
#[allow(invalid_type_param_default)]
pub const fn repeat_n_with<G, L>(repeater: G, n: L) -> RepeatNWith<G, L::Length<G::Output>>
where
    G: FnMut<()>,
    L: ~const LengthSpec
{
    RepeatNWith {
        repeater,
        n: n.len_metadata()
    }
}

/// A bulk that repeats elements of type `A` an exact number of times by
/// applying the provided closure `F: FnMut() -> A`.
///
/// This `struct` is created by the [`repeat_n_with()`] function.
/// See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct RepeatNWith<G, N = [<G as FnOnce<()>>::Output]>
where
    G: FnMut<()>,
    N: Length<Elem = G::Output> + ?Sized
{
    repeater: G,
    n: <N as Pointee>::Metadata
}

impl<A, G, N> fmt::Debug for RepeatNWith<G, N>
where
    G: FnMut() -> A,
    N: Length<Elem = A> + ?Sized
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("RepeatNWith").field("count", &self.len()).finish()
    }
}

impl<A, G, N> IntoIterator for RepeatNWith<G, N>
where
    G: FnMut() -> A,
    N: Length<Elem = A> + ?Sized
{
    type Item = A;
    type IntoIter = core::iter::Take<core::iter::RepeatWith<G>>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { repeater, n } = self;
        core::iter::repeat_with(repeater).take(N::len_metadata(n))
    }
}
impl<A, G, N> const Bulk for RepeatNWith<G, N>
where
    G: ~const FnMut() -> A + ~const Destruct,
    N: ~const Length<Elem = A> + ?Sized
{
    fn len(&self) -> usize
    {
        let Self { repeater: _, n } = self;
        N::len_metadata(*n)
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { mut repeater, n } = self;
        let n = N::len_metadata(n);
        let mut i = 0;
        while i < n
        {
            f(repeater());
            i += 1
        }
    }
    fn try_for_each<F, R>(self, mut f: F) -> R
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        let Self { mut repeater, n } = self;
        let n = N::len_metadata(n);
        let mut i = 0;
        while i < n
        {
            f(repeater())?;
            i += 1
        }
        R::from_output(())
    }
}
impl<A, G, N> const DoubleEndedBulk for RepeatNWith<G, N>
where
    G: ~const FnMut() -> A + ~const Destruct,
    N: ~const Length<Elem = A> + ?Sized,
    Self::IntoIter: DoubleEndedIterator
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
        A: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        self.try_for_each(f)
    }
}
impl<A, G, const N: usize> StaticBulk for RepeatNWith<G, [A; N]>
where
    G: FnMut() -> A
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
        let mut i = 0;
        let a = crate::repeat_n_with(|| {i += 1; i}, [(); _])
            .collect_array();
        assert_eq!(a, [1, 2, 3, 4])
    }
}