use core::fmt;

use crate::{Bulk, LimitToBulk, StaticBulk};

/// A bulk that maps the values of `bulk` with `f`.
///
/// This `struct` is created by the [`map`](Bulk::map) method on [`Bulk`]. See its
/// documentation for more.
///
/// # Notes about side effects
///
/// The [`map`](Bulk::map) iterator implements [`DoubleEndedIterator`](core::iter::DoubleEndedIterator), meaning that
/// you can also [`map`](Bulk::map) backwards:
///
/// ```rust
/// use bulks::*;
/// 
/// let v: [i32; 3] = [1, 2, 3].into_bulk().map(|x| x + 1).rev().collect();
///
/// assert_eq!(v, [4, 3, 2]);
/// ```
///
/// But if your closure has state, iterating backwards may act in a way you do
/// not expect. Let's go through an example. First, in the forward direction:
///
/// ```rust
/// use bulks::*;
/// 
/// let mut c = 0;
///
/// for pair in ['a', 'b', 'c'].into_bulk()
///     .map(|letter| { c += 1; (letter, c) })
/// {
///     println!("{pair:?}");
/// }
/// ```
///
/// This will print `('a', 1), ('b', 2), ('c', 3)`.
///
/// Now consider this twist where we add a call to [`rev`](Bulk::rev). This version will
/// print `('c', 1), ('b', 2), ('a', 3)`. Note that the letters are reversed,
/// but the values of the counter still go in order. This is because `map()` is
/// still being called lazily on each item, but we are popping items off the
/// back of the array now, instead of shifting them from the front.
///
/// ```rust
/// use bulks::*;
/// 
/// let mut c = 0;
///
/// for pair in ['a', 'b', 'c'].into_bulk()
///     .map(|letter| { c += 1; (letter, c) })
///     .rev()
/// {
///     println!("{pair:?}");
/// }
/// ```
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    bulk: I,
    f: F
}

impl<I, F> Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    pub(crate) fn new(bulk: I, f: F) -> Self
    {
        Self {
            bulk,
            f
        }
    }

    pub(crate) fn into_inner(self) -> I
    {
        let Self { bulk, f: _ } = self;
        bulk
    }
}

impl<I, F> fmt::Debug for Map<I, F>
where
    I: Bulk + fmt::Debug,
    F: FnMut<(I::Item,)>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let Self { bulk, f: _ } = self;
        f.debug_struct("Map").field("bulk", bulk).finish()
    }
}

impl<I, F> IntoIterator for Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    type Item = F::Output;
    type IntoIter = core::iter::Map<I::IntoIter, F>;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk, f } = self;
        bulk.into_iter().map(f)
    }
}
impl<I, F> LimitToBulk for Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    
}
impl<I, F> Bulk for Map<I, F>
where
    I: Bulk,
    F: FnMut<(I::Item,)>
{
    fn len(&self) -> usize
    {
        self.bulk.len()
    }

    fn is_empty(&self) -> bool
    {
        self.bulk.is_empty()
    }
}
impl<I, F, T, U, const N: usize> StaticBulk for Map<I, F>
where
    I: StaticBulk<Item = T, Array = [T; N]>,
    F: FnMut(T) -> U
{
    type Array = [Self::Item; N];

    fn collect_array(self) -> Self::Array
    {
        let Self { bulk, f } = self;
        bulk.map_collect_array(f)
    }
}

pub(crate) trait StaticMapSpec<const N: usize>: StaticBulk<Array = [<Self as IntoIterator>::Item; N]>
{
    fn map_collect_array<U>(self, f: impl FnMut(Self::Item) -> U) -> [U; N];
}

impl<T, I, const N: usize> StaticMapSpec<N> for I
where
    I: StaticBulk<Item = T, Array = [T; N]>
{
    default fn map_collect_array<U>(self, f: impl FnMut(T) -> U) -> [U; N]
    {
        self.into_iter().map(f).next_chunk().ok().unwrap()
    }
}