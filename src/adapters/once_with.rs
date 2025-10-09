use core::fmt;

use crate::{Bulk, StaticBulk};

/// Creates a bulk that lazily generates a value exactly once by invoking
/// the provided closure.
///
/// Unlike [`once()`](crate::once), this function will lazily generate the value on request.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// // one is the loneliest number
/// let mut one = bulks::once_with(|| 1).collect();
///
/// // just one, that's all we get
/// assert_eq!(one, [1])
/// ```
pub const fn once_with<A, F>(value: F) -> OnceWith<F>
where
    F: FnOnce() -> A
{
    OnceWith(value)
}

/// A bulk that yields a single element of type `A` by
/// applying the provided closure `F: FnOnce() -> A`.
///
/// This `struct` is created by the [`once_with()`] function.
/// See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct OnceWith<F>(F)
where
    F: FnOnce<()>;

impl<F, A> fmt::Debug for OnceWith<F>
where
    F: FnOnce() -> A
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.write_str("OnceWith")
    }
}

impl<F, A> IntoIterator for OnceWith<F>
where
    F: FnOnce() -> A
{
    type Item = A;
    type IntoIter = core::iter::OnceWith<F>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::once_with(self.0)
    }
}
impl<F, A> Bulk for OnceWith<F>
where
    F: FnOnce() -> A
{
    fn len(&self) -> usize
    {
        1
    }
}
impl<F, A> StaticBulk for OnceWith<F>
where
    F: FnOnce() -> A
{
    type Array = [A; 1];

    fn collect_array(self) -> Self::Array
    {
        [self.0()]
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::once_with(|| 1).collect::<[_; _]>();
        assert_eq!(a, [1])
    }
}