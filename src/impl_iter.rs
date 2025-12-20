use core::{marker::Destruct, ops::Try};

use array_trait::{length::{self, LengthValue}, same::Same};

use crate::{Bulk, DoubleEndedBulk, IntoBulk, RandomAccessBulk, Step, range::BoundedRange};

pub mod iter
{
    pub struct Bulk<T>
    where
        T: IntoIterator<IntoIter: ExactSizeIterator>
    {
        pub(super) iter: T::IntoIter
    }
}

impl<T, A, I> iter::Bulk<T>
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A>
{
    pub const fn by_ref(&mut self) -> &mut T::IntoIter
    {
        let Self { iter } = self;
        iter
    }
}

impl<T, A, I> IntoIterator for iter::Bulk<T>
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A>
{
    type IntoIter = I;
    type Item = A;
    
    fn into_iter(self) -> Self::IntoIter
    {
        self.iter
    }
}

impl<T, A, I> IntoBulk for T
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A>
{
    default type IntoBulk = iter::Bulk<T>;

    default fn into_bulk(self) -> Self::IntoBulk
    {
        iter::Bulk::<T> {
            iter: self.into_iter()
        }.same().ok().unwrap()
    }
}

impl<T, A, I> Bulk for iter::Bulk<T>
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A>
{
    #[inline]
    default fn len(&self) -> usize
    {
        self.iter.len()
    }
    #[inline]
    default fn is_empty(&self) -> bool
    {
        self.iter.is_empty()
    }

    #[inline]
    default fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.iter.for_each(f)
    }
    #[inline]
    default fn try_for_each<F, R>(mut self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: core::ops::Try<Output = ()>
    {
        self.iter.try_for_each(f)
    }

    default fn first(mut self) -> Option<Self::Item>
    where
        Self: Sized
    {
        self.iter.next()
    }
    
    default fn nth<L>(mut self, n: L) -> Option<Self::Item>
    where
        Self: Sized,
        L: LengthValue
    {
        self.iter.nth(length::value::len(n))
    }
}
impl<T, A, I> DoubleEndedBulk for iter::Bulk<T>
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A> + DoubleEndedIterator
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.iter.rev().for_each(f);
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.iter.rev().try_for_each(f)
    }
}

impl<R> const Bulk for iter::Bulk<R>
where
    R: ~const BoundedRange<R::Item> + ExactSizeIterator<Item: Copy + ~const Step> + ~const Destruct,
{
    fn len(&self) -> usize
    {
        self.iter.steps().0
    }
    fn first(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        if !self.is_empty()
        {
            return Some(*self.iter.start())
        }
        None
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { iter } = self;
        let inclusive = iter.inclusive();
        let mut range = *iter.start()..*iter.end();
        loop
        {
            let n = Step::steps_between(&range.start, &range.end).0;
            if n == 0
            {
                if inclusive
                {
                    f(range.start)
                }
                break
            }

            f(range.start);
            range.start = Step::forward(range.start, 1);
        }
    }
    fn try_for_each<F, RR>(self, mut f: F) -> RR
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) -> RR + ~const Destruct,
        RR: ~const Try<Output = ()>
    {
        let Self { iter } = self;
        let inclusive = iter.inclusive();
        let mut range = *iter.start()..*iter.end();
        loop
        {
            let n = Step::steps_between(&range.start, &range.end).0;
            if n == 0
            {
                if inclusive
                {
                    f(range.start)?
                }
                break RR::from_output(())
            }
            
            f(range.start)?;
            range.start = Step::forward(range.start, 1);
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn vec()
    {
        let a = vec![1i32, 2, 3, 4, 5];
        
        let bulk = a.into_bulk().map(|x| x as f64);

        let b: Vec<f64> = bulk.collect();

        println!("{b:?}")
    }
}