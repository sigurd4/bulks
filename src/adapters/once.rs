use core::marker::Destruct;

use crate::{Bulk, DoubleEndedBulk, StaticBulk};

/// Creates a bulk that yields an element exactly once.
/// 
/// Similar to [`core::iter::once`].
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
    fn is_empty(&self) -> bool
    {
        false
    }
    fn for_each<FF>(self, mut f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        f(self.0)
    }
    fn try_for_each<FF, R>(self, mut f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        f(self.0)
    }
}
impl<T> const DoubleEndedBulk for Once<T>
{
    fn rev_for_each<FF>(self, f: FF)
    where
        Self: Sized,
        FF: ~const FnMut(Self::Item) + ~const Destruct
    {
        self.for_each(f);
    }
    fn try_rev_for_each<FF, R>(self, f: FF) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        FF: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const core::ops::Try<Output = (), Residual: ~const Destruct>
    {
        self.try_for_each(f)
    }
}
impl<T> StaticBulk for Once<T>
{
    type Array<U> = [U; 1];
}

pub const trait OnceBulk: ~const DoubleEndedBulk + StaticBulk<Array<<Self as IntoIterator>::Item> = [<Self as IntoIterator>::Item; 1]>
{

}
impl<T> const OnceBulk for T
where
    T: ~const DoubleEndedBulk + StaticBulk<Array<<Self as IntoIterator>::Item> = [<Self as IntoIterator>::Item; 1]>
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