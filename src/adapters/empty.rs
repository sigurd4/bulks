use core::{fmt, marker::PhantomData};

use crate::{Bulk, IntoBulk, StaticBulk};

/// Creates a bulk that yields nothing.
/// 
/// Analogous to [`core::iter::empty`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
///
/// let mut nope = bulks::empty::<i32>();
///
/// let nothing = nope.collect();
/// 
/// assert_eq!(nothing, []);
/// ```
pub const fn empty<T>() -> Empty<T>
{
    Empty(PhantomData)
}

/// A bulk that yields nothing.
///
/// This `struct` is created by the [`empty()`] function. See its documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Empty<T>(PhantomData<T>);

impl<T> fmt::Debug for Empty<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Empty").finish()
    }
}

impl<T> IntoIterator for Empty<T>
{
    type Item = T;
    type IntoIter = core::iter::Empty<T>;

    fn into_iter(self) -> Self::IntoIter
    {
        core::iter::empty()
    }
}
impl<T> IntoBulk for core::iter::Empty<T>
{
    type IntoBulk = Empty<T>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        empty()
    }
}
impl<T> Bulk for Empty<T>
{
    fn len(&self) -> usize
    {
        0
    }
}
impl<T> StaticBulk for Empty<T>
{
    type Array = [T; 0];

    fn collect_array(self) -> Self::Array
    {
        []
    }
}

pub trait EmptyBulk: StaticBulk<Array = [<Self as IntoIterator>::Item; 0]>
{

}
impl<T> EmptyBulk for T
where
    T: StaticBulk<Array = [<Self as IntoIterator>::Item; 0]>
{

}

#[cfg(test)]
mod test
{
    use crate::Bulk;

    #[test]
    fn it_works()
    {
        let a = crate::empty::<u8>().collect::<[_; _]>();
        assert_eq!(a, [])
    }
}