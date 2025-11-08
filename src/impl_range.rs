use core::{marker::Destruct, ops::{Range, RangeInclusive, Try}};

use crate::{Bulk, IntoBulk, IntoContainedIter, Step, range::BoundedRange};

pub mod range
{
    use core::{ops::{Bound, Range, RangeBounds, RangeInclusive}};

    use currying::RCurry;

    use crate::Step;

    pub struct IntoBulk<R>
    where
        R: BoundedRange<R::Item> + IntoIterator
    {
        pub(super) range: R
    }

    pub const trait BoundedRange<T>: ~const RangeBounds<T>
    {
        fn steps(&self) -> (usize, Option<usize>)
        where
            T: ~const Step;
        fn start(&self) -> &T;
        fn end(&self) -> &T;
        fn last(&self) -> T
        where
            T: ~const Step + Copy;
        fn inclusive(&self) -> bool
        {
            matches!(self.end_bound(), Bound::Included(_))
        }
    }
    impl<T> const BoundedRange<T> for Range<T>
    {
        fn steps(&self) -> (usize, Option<usize>)
        where
            T: ~const Step
        {
            Step::steps_between(self.start(), self.end())
        }
        fn start(&self) -> &T
        {
            &self.start
        }
        fn end(&self) -> &T
        {
            &self.end
        }
        fn last(&self) -> T
        where
            T: ~const Step + Copy
        {
            self.end
        }
        fn inclusive(&self) -> bool
        {
            false
        }
    }
    impl<T> const BoundedRange<T> for RangeInclusive<T>
    {
        fn steps(&self) -> (usize, Option<usize>)
        where
            T: ~const Step
        {
            let (n, o) = Step::steps_between(self.start(), self.end());
            (n.saturating_add(1), o.and_then(usize::checked_add.rcurry(1)))
        }
        fn start(&self) -> &T
        {
            self.start()
        }
        fn end(&self) -> &T
        {
            self.end()
        }
        fn last(&self) -> T
        where
            T: ~const Step + Copy
        {
            Step::backward(*self.end(), 1)
        }
    }
}

impl<R> IntoIterator for range::IntoBulk<R>
where
    R: BoundedRange<R::Item> + IntoIterator
{
    type Item = R::Item;
    type IntoIter = <R as IntoContainedIter>::IntoContainedIter;

    fn into_iter(self) -> Self::IntoIter
    {
        unsafe {
            self.range.into_contained_iter()
        }
    }
}


impl<T> const IntoBulk for Range<T>
where
    Self: ~const BoundedRange<T> + IntoIterator<Item = T, IntoIter: ExactSizeIterator> + ~const Destruct,
    T: Copy + ~const Step
{
    type IntoBulk = range::IntoBulk<Self>;

    fn into_bulk(self) -> Self::IntoBulk
    {
        range::IntoBulk {
            range: self
        }
    }
}
impl<T> const IntoBulk for RangeInclusive<T>
where
    Self: ~const BoundedRange<T> + IntoIterator<Item = T, IntoIter: ExactSizeIterator> + ~const Destruct,
    T: Copy + ~const Step
{
    type IntoBulk = range::IntoBulk<Self>;

    fn into_bulk(self) -> Self::IntoBulk
    {
        range::IntoBulk {
            range: self
        }
    }
}

impl<R> const Bulk for range::IntoBulk<R>
where
    R: ~const BoundedRange<R::Item> + IntoIterator<Item: Copy + ~const Step> + ~const Destruct,
{
    fn len(&self) -> usize
    {
        self.range.steps().0
    }
    fn first(self) -> Option<Self::Item>
    where
        Self: Sized
    {
        if !self.is_empty()
        {
            return Some(*self.range.start())
        }
        None
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        let Self { range } = self;
        let inclusive = range.inclusive();
        let mut range = *range.start()..*range.end();
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
        let Self { range } = self;
        let inclusive = range.inclusive();
        let mut range = *range.start()..*range.end();
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
    use crate::*;

    #[test]
    fn test()
    {
        let r = 0usize..4;
        let r = r.into_bulk()
            .map(|n| n + 1)
            .collect::<Vec<_>>();
        
        assert_eq!(r, [1, 2, 3, 4]);

        let r = 0u16..=4;
        let r = r.into_bulk()
            .map(|n| n + 1)
            .collect::<Vec<_>>();
        
        assert_eq!(r, [1, 2, 3, 4, 5]);
    }
}