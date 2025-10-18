use crate::{Bulk, IntoContained, StaticBulk};

/// A bulk adapter that places a separator between all elements.
///
/// This `struct` is created by [`Bulk::intersperse`]. See its documentation
/// for more information.
#[derive(Debug, Clone)]
pub struct Intersperse<I>
where
    I: Bulk<Item: Clone>
{
    bulk: I,
    separator: I::Item
}

impl<I, T> Intersperse<I>
where
    I: Bulk<Item = T>,
    T: Clone
{
    pub(crate) const fn new(bulk: I, separator: I::Item) -> Self
    {
        Self {
            bulk,
            separator
        }
    }
}

impl<I, T> IntoIterator for Intersperse<I>
where
    I: Bulk<Item = T>,
    T: Clone
{
    type Item = I::Item;
    type IntoIter = <<core::iter::Intersperse<I::IntoIter> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, separator } = self;
        unsafe {
            bulk.into_iter()
                .intersperse(separator)
                .into_contained()
                .into_iter()
        }
    }
}
impl<I, T> const Bulk for Intersperse<I>
where
    I: ~const Bulk<Item = T>,
    T: Clone
{
    fn len(&self) -> usize
    {
        let Self { bulk, separator: _ } = self;
        let l = bulk.len();
        l + l.saturating_sub(1)
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk, separator: _ } = self;
        bulk.is_empty()
    }
}
impl<I, T, const N: usize> StaticBulk for Intersperse<I>
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    T: Clone,
    [(); N + N.saturating_sub(1)]:
{
    type Array = [T; N + N.saturating_sub(1)];

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
        let a = ['H', 'e', 'l', 'l', 'o'];
        let b = '_';
        let c = a.into_bulk().intersperse(b).collect::<String>();

        println!("{:?}", c);
    }
}