use core::fmt;

use crate::{Bulk, StaticBulk};

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
pub const fn repeat_n<T, const N: usize>(element: T) -> RepeatN<T, N>
where
    T: Clone
{
    RepeatN {
        element
    }
}

/// A bulk that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function. See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct RepeatN<A, const N: usize>
where
    A: Clone
{
    element: A
}

impl<A, const N: usize> fmt::Debug for RepeatN<A, N>
where
    A: Clone + fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("RepeatN").field("count", &N).field("element", &self.element).finish()
    }
}

impl<A, const N: usize> IntoIterator for RepeatN<A, N>
where
    A: Clone
{
    type Item = A;
    type IntoIter = core::iter::RepeatN<A>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::repeat_n(self.element, N)
    }
}
impl<A, const N: usize> Bulk for RepeatN<A, N>
where
    A: Clone
{
    fn len(&self) -> usize
    {
        N
    }
}
impl<A, const N: usize> StaticBulk for RepeatN<A, N>
where
    A: Clone
{
    type Array = [A; N];

    fn collect_array(self) -> Self::Array
    {
        self.element.repeat()
    }
}

trait RepeatSpec: Clone
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
impl<A> RepeatSpec for A
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
        let a = crate::repeat_n(1).collect::<[_; _]>();
        assert_eq!(a, [1, 1, 1, 1])
    }
}