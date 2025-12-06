use array_trait::length::LengthValue;

use crate::Bulk;

pub const trait SplitBulk<L>: ~const Bulk
where
    L: LengthValue
{
    type Left: Bulk<Item = Self::Item>;
    type Right: Bulk<Item = Self::Item>;

    /// Splits a bulk in two at a specified index.
    /// 
    /// # Example
    /// 
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let a = b"leftright";
    /// 
    /// let (a1, a2) = a.bulk()
    ///     .copied()
    ///     .split_at([(); 4]);
    /// 
    /// let left: [_; _] = a1.collect();
    /// let right: [_; _] = a2.collect();
    /// 
    /// assert_eq!(&left, b"left");
    /// assert_eq!(&right, b"right");
    /// ```
    #[track_caller]
    fn split_at(bulk: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized;
}