pub mod range
{
    use core::ops::{Bound, Range, RangeBounds, RangeInclusive};

    use currying::RCurry;

    use crate::Step;

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

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn test()
    {
        let r = 0..4;
        let r = r.into_bulk()
            .map(|n| n + 1)
            .collect::<Vec<_>, _>();
        
        assert_eq!(r, [1, 2, 3, 4]);

        let r = 0u16..=4;
        let r = r.into_bulk()
            .map(|n| n + 1)
            .collect::<Vec<_>, _>();
        
        assert_eq!(r, [1, 2, 3, 4, 5]);
    }
}