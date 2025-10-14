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
pub const fn once<T>(value: T) -> Once<T>
{
    Once(value)
}

/// A bulk that yields an element exactly once.
///
/// This `struct` is created by the [`once()`] function. See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Once<T>(T);

impl<T> Once<T>
{
    const fn into_inner(self) -> T
    {
        crate::const_inner!(
            for<{T}> Once (x) in self => Once<T> => T
            {
                x
            }
        )
    }
}

impl<T> IntoIterator for Once<T>
{
    type Item = T;
    type IntoIter = core::iter::Once<T>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::once(self.0)
    }
}
impl<T> const Bulk for Once<T>
{
    fn len(&self) -> usize
    {
        1
    }
}
impl<T> const StaticBulk for Once<T>
{
    type Array = [T; 1];

    fn collect_array(self) -> Self::Array
    {
        [self.into_inner()]
    }
}

pub const trait OnceBulk: ~const StaticBulk<Array = [<Self as IntoIterator>::Item; 1]>
{

}
impl<T> const OnceBulk for T
where
    T: ~const StaticBulk<Array = [<Self as IntoIterator>::Item; 1]>
{

}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = const {
            crate::once(1).collect::<[_; _]>()
        };
        assert_eq!(a, [1])
    }
}