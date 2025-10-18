use core::mem::{ManuallyDrop, MaybeUninit};

use crate::{Bulk, Contained, ContainedIntoIter, IntoBulk, IntoContained, StaticBulk};


/// A bulk that links two bulks together, in a chain.
///
/// This `struct` is created by [`chain`] or [`Bulk::chain`]. See their
/// documentation for more.
///
/// # Examples
///
/// ```
/// use bulks::{*, array::Bulk};
///
/// let a1 = [1, 2, 3];
/// let a2 = [4, 5, 6];
/// let bulk: Chain<Bulk<'_, _, _>, Bulk<'_, _, _>> = a1.bulk().chain(a2.bulk());
/// 
/// let a = bulk.collect();
/// 
/// assert_eq!(a, [1, 2, 3, 4, 5, 6]);
/// ```
#[derive(Clone, Debug)]
#[must_use = "bulks are lazy and do nothing unless consumed"]
pub struct Chain<A, B>
where
    A: Bulk,
    B: Bulk<Item = A::Item>
{
    a: A,
    b: B
}
impl<A, B> Chain<A, B>
where
    A: Bulk,
    B: Bulk<Item = A::Item>
{
    pub(crate) const fn new(a: A, b: B) -> Self
    {
        Self { a, b }
    }
}

/// Converts the arguments to bulks and links them together, in a chain.
///
/// See the documentation of [`Bulk::chain`] for more.
///
/// # Examples
///
/// ```
/// use bulks::*;
///
/// let a = [1, 2, 3];
/// let b = [4, 5, 6];
///
/// let mut bulk = bulks::chain(a, b);
///
/// let c = bulk.collect();
/// 
/// assert_eq!(c, [1, 2, 3, 4, 5, 6]);
/// ```
pub const fn chain<A, B>(a: A, b: B) -> Chain<A::IntoBulk, B::IntoBulk>
where
    A: ~const IntoBulk,
    B: ~const IntoBulk<Item = A::Item>
{
    Chain::new(a.into_bulk(), b.into_bulk())
}

impl<A, B, T> IntoIterator for Chain<A, B>
where
    A: Bulk<Item = T>,
    B: Bulk<Item = T>
{
    type Item = T;
    type IntoIter = <<core::iter::Chain<A::IntoIter, B::IntoIter> as IntoContained>::IntoContained as IntoIterator>::IntoIter;
    
    fn into_iter(self) -> Self::IntoIter
    {
        let Self { a, b } = self;
        unsafe {
            a.into_iter()
                .chain(b)
                .into_contained()
                .into_iter()
        }
    }
}
impl<A, B, T> const Bulk for Chain<A, B>
where
    A: ~const Bulk<Item = T>,
    B: ~const Bulk<Item = T>
{
    fn len(&self) -> usize
    {
        let Self { a, b } = self;
        a.len() + b.len()
    }
    fn is_empty(&self) -> bool
    {
        let Self { a, b } = self;
        a.is_empty() && b.is_empty()
    }
}
impl<A, B, T, const N: usize, const M: usize> const StaticBulk for Chain<A, B>
where
    A: ~const StaticBulk<Item = T, Array = [T; N]> + ~const StaticChainSpec<N, M, B>,
    B: ~const StaticBulk<Item = T, Array = [T; M]>,
    [(); N + M]:
{
    type Array = [T; N + M];

    fn collect_array(self) -> Self::Array
    {
        A::chain_collect_array(self)
    }
}

const trait StaticChainSpec<const N: usize, const M: usize, Rhs>: StaticBulk<Array = [<Self as IntoIterator>::Item; N]>
where
    Rhs: StaticBulk<Array = [Self::Item; M], Item = Self::Item>,
    [(); N + M]:
{
    fn chain_collect_array(chain: Chain<Self, Rhs>) -> [Self::Item; N + M];
}
impl<A, B, T, const N: usize, const M: usize> StaticChainSpec<N, M, B> for A
where
    A: StaticBulk<Item = T, Array = [T; N]>,
    B: StaticBulk<Item = T, Array = [T; M]>,
    [(); N + M]:
{
    default fn chain_collect_array(chain: Chain<Self, B>) -> [Self::Item; N + M]
    {
        chain.into_iter().next_chunk().ok().unwrap()
    }
}
impl<A, B, T, const N: usize, const M: usize> const StaticChainSpec<N, M, B> for A
where
    A: const StaticBulk<Item = T, Array = [T; N]>,
    B: const StaticBulk<Item = T, Array = [T; M]>,
    [(); N + M]:
{
    fn chain_collect_array(chain: Chain<Self, B>) -> [T; N + M]
    {
        fn rt<A, B, T, const N: usize, const M: usize>(chain: Chain<A, B>) -> [T; N + M]
        where
            A: StaticBulk<Item = T, Array = [T; N]>,
            B: StaticBulk<Item = T, Array = [T; M]>,
            [(); N + M]:
        {
            chain.into_iter().next_chunk().ok().unwrap()
        }
        const fn ct<A, B, T, const N: usize, const M: usize>(chain: Chain<A, B>) -> [T; N + M]
        where
            A: const StaticBulk<Item = T, Array = [T; N]>,
            B: const StaticBulk<Item = T, Array = [T; M]>,
            [(); N + M]:
        {
            #[repr(C)]
            struct Pair<A, B>
            {
                a: A,
                b: B
            }

            union Concat<T, const N: usize, const M: usize>
            where
                [(); N + M]:
            {
                pair: ManuallyDrop<Pair<[T; N], [T; M]>>,
                concat: ManuallyDrop<[T; N + M]>
            }

            let Chain { a, b } = &chain;
            let a = unsafe {
                core::ptr::read(a)
            };
            let b = unsafe {
                core::ptr::read(b)
            };
            core::mem::forget(chain);
            let ab = Concat {
                pair: ManuallyDrop::new(Pair {
                    a: a.collect_array(),
                    b: b.collect_array()
                })
            };
            unsafe {
                ManuallyDrop::into_inner(ab.concat)
            }
        }
        
        core::intrinsics::const_eval_select((chain,), ct, rt)
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let c = const {
            let a = [1, 2, 3];
            let b = [4, 5, 6];
            
            a.into_bulk().chain(b).collect::<[_; _]>()
        };

        println!("{c:?}")
    }
}