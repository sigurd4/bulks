use array_trait::AsArray;

use crate::{Bulk, IntoBulk, IntoContained, StaticBulk};

/// A bulk that maps each element to an iterator, and yields the elements
/// of the produced bulks.
///
/// This `struct` is created by [`Bulk::flat_map`]. See its documentation
/// for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct FlatMap<I, U, F>
where
    I: Bulk,
    F: FnMut(I::Item) -> U,
    U: IntoBulk<IntoBulk: StaticBulk>
{
    bulk: I,
    map: F
}

impl<I, U, F> FlatMap<I, U, F>
where
    I: Bulk,
    F: FnMut(I::Item) -> U,
    U: IntoBulk<IntoBulk: StaticBulk>
{
    pub(crate) const fn new(bulk: I, map: F) -> Self
    {
        Self {
            bulk,
            map
        }
    }
}

impl<I, U, F> IntoIterator for FlatMap<I, U, F>
where
    I: Bulk,
    F: FnMut(I::Item) -> U,
    U: IntoBulk<IntoBulk: StaticBulk>
{
    type Item = U::Item;
    type IntoIter = <<core::iter::FlatMap<I::IntoIter, U, F> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, map } = self;
        unsafe {
            bulk.into_iter()
                .flat_map(map)
                .into_contained()
                .into_iter()
        }
    }
}
impl<I, U, F> Bulk for FlatMap<I, U, F>
where
    I: Bulk,
    F: FnMut(I::Item) -> U,
    U: IntoBulk<IntoBulk: StaticBulk>
{
    fn len(&self) -> usize
    {
        self.bulk.len()*<<U::IntoBulk as StaticBulk>::Array as AsArray>::LENGTH
    }
    fn is_empty(&self) -> bool
    {
        <<U::IntoBulk as StaticBulk>::Array as AsArray>::LENGTH == 0 || self.bulk.is_empty()
    }
}
impl<I, U, F, T, V, const N: usize, const M: usize> StaticBulk for FlatMap<I, U, F>
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    F: FnMut(T) -> U,
    U: IntoBulk<Item = V, IntoBulk: StaticBulk<Item = V, Array = [V; M]>>,
    [(); N*M]:
{
    type Array = [V; N*M];

    fn collect_array(self) -> Self::Array
    {
        let Self { bulk, map } = self;
        bulk.into_iter()
            .flat_map(map)
            .next_chunk()
            .ok()
            .unwrap()
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3];
        let b = a.into_bulk()
            .flat_map(|x| [x, -x])
            .collect::<[_; _]>();

        println!("{b:?}")
    }
}