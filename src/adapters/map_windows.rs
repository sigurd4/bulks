use crate::{Bulk, StaticBulk};

/// A bulk over the mapped windows of another bulk.
///
/// This `struct` is created by the [`Bulk::map_windows`]. See its
/// documentation for more information.
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct MapWindows<I, F, const N: usize>
where
    I: Bulk,
    F: for<'a> FnMut<(&'a [I::Item; N],)>
{
    bulk: I,
    f: F,
}

impl<I: Bulk, F, U, const N: usize> MapWindows<I, F, N>
where
    I: Bulk,
    F: FnMut(&[I::Item; N]) -> U
{
    pub(crate) const fn new(bulk: I, f: F) -> Self
    {
        assert!(N != 0, "array in `Bulk::map_windows` must contain more than 0 elements");

        // Only ZST arrays' length can be so large.
        if core::mem::size_of::<I::Item>() != 0
        {
            assert!(
                N.checked_mul(2).is_some(),
                "array size of `Iterator::map_windows` is too large"
            );
        }

        Self {
            bulk,
            f
        }
    }
}

impl<I: Bulk, F, U, const N: usize> IntoIterator for MapWindows<I, F, N>
where
    I: Bulk,
    F: FnMut(&[I::Item; N]) -> U
{
    type Item = U;
    type IntoIter = core::iter::MapWindows<I::IntoIter, F, N>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter()
            .map_windows(f)
    }
}

impl<I: Bulk, F, U, const N: usize> Bulk for MapWindows<I, F, N>
where
    I: Bulk,
    F: FnMut(&[I::Item; N]) -> U
{
    fn len(&self) -> usize
    {
        let Self { bulk, f: _ } = self;
        bulk.len().saturating_sub(N - 1)
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, f: _ } = self;
        bulk.len() > N - 1
    }
}

impl<I: Bulk, F, T, U, const N: usize, const M: usize> StaticBulk for MapWindows<I, F, N>
where
    I: StaticBulk<Item = T, Array = [T; M]>,
    F: FnMut(&[T; N]) -> U,
    [(); M.saturating_sub(N - 1)]:
{
    type Array = [U; M.saturating_sub(N - 1)];

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

        let b = a.into_bulk()
            .map_windows(|&[n, m]| n + m)
            .collect::<[_; _]>();

        println!("{b:?}")
    }
}