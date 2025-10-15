use crate::{Bulk, StaticBulk};

/// A bulk over `N` elements of the bulk at a time.
///
/// The chunks do not overlap. If `N` does not divide the length of the
/// iterator, then the last up to `N-1` elements will be omitted.
///
/// This `struct` is created by the [`array_chunks`][Bulk::array_chunks]
/// method on [`Bulk`]. See its documentation for more.
#[derive(Debug, Clone)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct ArrayChunks<I, const N: usize>
where
    I: Bulk
{
    bulk: I
}

impl<I, const N: usize> ArrayChunks<I, N>
where
    I: Bulk
{
    #[track_caller]
    pub(crate) const fn new(bulk: I) -> Self
    {
        assert!(N != 0, "chunk size must be non-zero");
        Self {
            bulk
        }
    }

    /// Returns an iterator over the remaining elements of the original bulk
    /// that are not going to be returned by this bulk. The returned
    /// iterator will yield at most `N-1` elements.
    ///
    /// # Example
    /// ```
    /// use bulks::*;
    /// 
    /// let x = [1, 2, 3, 4, 5].into_bulk().array_chunks::<2>();
    /// let mut rem = x.into_remainder().unwrap();
    /// assert_eq!(rem.next(), Some(5));
    /// assert_eq!(rem.next(), None);
    /// ```
    #[inline]
    pub fn into_remainder(self) -> Option<core::array::IntoIter<I::Item, N>>
    {
        self.into_iter().into_remainder()
    }
}

impl<I, const N: usize> IntoIterator for ArrayChunks<I, N>
where
    I: Bulk
{
    type Item = [I::Item; N];
    type IntoIter = core::iter::ArrayChunks<I::IntoIter, N>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.bulk.into_iter().array_chunks()
    }
}
impl<I, const N: usize> Bulk for ArrayChunks<I, N>
where
    I: Bulk,
{
    #[inline]
    fn len(&self) -> usize
    {
        self.bulk.len()/N
    }
}
impl<I, T, const N: usize, const M: usize> StaticBulk for ArrayChunks<I, N>
where
    I: StaticBulk<Item = T, Array = [T; M]>,
    [(); M/N]:
{
    type Array = [[T; N]; M/N];

    fn collect_array(self) -> Self::Array
    {
        self.into_iter().next_chunk().ok().unwrap()
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3, 4];

        let b = a.into_bulk().array_chunks::<2>().collect::<[_; _]>();

        println!("{b:?}")
    }
}