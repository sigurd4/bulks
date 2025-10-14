use core::fmt;

use crate::{Bulk, Once, StaticBulk};

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

impl<F, A> OnceWith<F>
where
    F: FnOnce() -> A
{
    const fn into_inner(self) -> F
    {
        crate::const_inner!(
            for<{F, A}> OnceWith (f) in self => OnceWith<F> => F
            where {
                F: FnOnce() -> A
            }
            {
                f
            }
        )
    }
}

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
impl<F, A> const Bulk for OnceWith<F>
where
    F: FnOnce() -> A
{
    fn len(&self) -> usize
    {
        1
    }
}
impl<F, A> const StaticBulk for OnceWith<F>
where
    F: ~const FnOnce() -> A
{
    type Array = [A; 1];

    default fn collect_array(self) -> Self::Array
    {
        Once::from(self).collect_array()
    }
}
impl<F, A> const From<OnceWith<F>> for Once<A>
where
    F: ~const FnOnce() -> A
{
    fn from(value: OnceWith<F>) -> Self
    {
        crate::once(value.into_inner()())
    }
}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        const fn one() -> i32
        {
            1
        }

        let a = const {
            crate::once_with(one).collect::<[_; _]>()
        };
        assert_eq!(a, [1])
    }
}