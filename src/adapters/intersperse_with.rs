use crate::{Bulk, IntoContained, StaticBulk};

/// A bulk adapter that places a separator between all elements.
///
/// This `struct` is created by [`Bulk::intersperse_with`]. See its
/// documentation for more information.
#[derive(Debug, Clone)]
pub struct IntersperseWith<I, G>
where
    I: Bulk,
    G: FnMut() -> I::Item
{
    bulk: I,
    separator: G
}

impl<I, G, T> IntersperseWith<I, G>
where
    I: Bulk<Item = T>,
    G: FnMut() -> T
{
    pub(crate) const fn new(bulk: I, separator: G) -> Self
    {
        Self {
            bulk,
            separator
        }
    }
}

impl<I, G, T> IntoIterator for IntersperseWith<I, G>
where
    I: Bulk<Item = T>,
    G: FnMut() -> T
{
    type Item = I::Item;
    type IntoIter = <<core::iter::IntersperseWith<I::IntoIter, G> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, separator } = self;
        unsafe {
            bulk.into_iter()
                .intersperse_with(separator)
                .into_contained()
                .into_iter()
        }
    }
}
impl<I, G, T> const Bulk for IntersperseWith<I, G>
where
    I: ~const Bulk<Item = T>,
    G: FnMut() -> T
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
impl<I, G, T, const N: usize> StaticBulk for IntersperseWith<I, G>
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    G: FnMut() -> T,
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
        let c = a.into_bulk().intersperse_with(|| b).collect::<String>();

        println!("{:?}", c);
    }
}