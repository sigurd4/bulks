use core::{fmt, ptr::Pointee};

use crate::{util::Length, Bulk, StaticBulk};

/// Creates a bulk that yields an element exactly once.
/// 
/// Analogous to [`core::iter::once`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// // one is the loneliest number
/// let mut one = bulks::once(1).collect();
///
/// // just one, that's all we get
/// assert_eq!(one, [1])
/// ```
#[allow(invalid_type_param_default)]
pub const fn repeat_n<T, N = [T]>(element: T, n: <N as Pointee>::Metadata) -> RepeatN<T, N>
where
    T: Clone,
    N: Length<Elem = T> + ?Sized
{
    RepeatN {
        element,
        n
    }
}

/// A bulk that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function. See its documentation for more.
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

impl<A, N> RepeatN<A, N>
where
    A: Clone,
    N: Length<Elem = A> + ?Sized
{
    const fn into_inner(self) -> A
    {
        crate::const_inner!(
            for<{A, N}> RepeatN {element, n: _} in self => RepeatN<A, N> => A
            where {
                A: Clone,
                N: Length<Elem = A> + ?Sized
            }
            {
                element
            }
        )
    }
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
        let n = self.len();
        core::iter::repeat_n(self.element, n)
    }
}
impl<A, N> const Bulk for RepeatN<A, N>
where
    A: Clone,
    N: ~const Length<Elem = A> + ?Sized
{
    fn len(&self) -> usize
    {
        let Self { element: _, n } = self;
        N::len_metadata(*n)
    }
}
impl<A, const N: usize> const StaticBulk for RepeatN<A, [A; N]>
where
    A: ~const RepeatSpec
{
    type Array = [A; N];

    fn collect_array(self) -> Self::Array
    {
        self.into_inner().repeat()
    }
}

const trait RepeatSpec: Clone
{
    fn repeat<const N: usize>(self) -> [Self; N];
}
impl<A> RepeatSpec for A
where
    A: Clone
{
    default fn repeat<const N: usize>(self) -> [Self; N]
    {
        core::array::repeat(self)
    }
}
impl<A> const RepeatSpec for A
where
    A: Copy
{
    fn repeat<const N: usize>(self) -> [Self; N]
    {
        [self; _]
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::repeat_n(1, ()).collect::<[_; _]>();
        assert_eq!(a, [1, 1, 1, 1])
    }
}