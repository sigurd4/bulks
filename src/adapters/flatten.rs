use array_trait::AsArray;

use crate::{Bulk, IntoBulk, IntoContained, StaticBulk};

/// A bulk that flattens one level of nesting in a of things
/// that can be turned into bulks.
///
/// This `struct` is created by the [`flatten`](Bulk::flatten) method on [`Bulk`]. See its
/// documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    bulk: I
}

impl<I> Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    pub(crate) const fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }
}

impl<I> IntoIterator for Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    type Item = <I::Item as IntoIterator>::Item;
    type IntoIter = <<core::iter::Flatten<I::IntoIter> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk } = self;
        unsafe {
            bulk.into_iter()
                .flatten()
                .into_contained()
                .into_iter()
        }
    }
}
impl<I> Bulk for Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    fn len(&self) -> usize
    {
        self.bulk.len()*<<<I::Item as IntoBulk>::IntoBulk as StaticBulk>::Array as AsArray>::LENGTH
    }
    fn is_empty(&self) -> bool
    {
        <<<I::Item as IntoBulk>::IntoBulk as StaticBulk>::Array as AsArray>::LENGTH == 0 || self.bulk.is_empty()
    }
}
impl<I, T, V, const N: usize, const M: usize> StaticBulk for Flatten<I>
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    T: IntoBulk<Item = V, IntoBulk: StaticBulk<Item = V, Array = [V; M]>>,
    [(); N*M]:
{
    type Array = [V; N*M];

    fn collect_array(self) -> Self::Array
    {
        let Self { bulk } = self;
        bulk.into_iter()
            .flatten()
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
        let a = [[1, -1], [2, -2], [3, -3]];
        let b = a.into_bulk()
            .flatten()
            .collect::<[_; _]>();

        println!("{b:?}")
    }
}