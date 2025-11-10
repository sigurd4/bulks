use array_trait::length::LengthValue;

use crate::Bulk;

pub const trait SplitBulk<L>: Bulk
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
    /// let left = a1.collect::<[_; _]>();
    /// let right = a2.collect::<[_; _]>();
    /// 
    /// assert_eq!(&left, b"left");
    /// assert_eq!(&right, b"right");
    /// ```
    #[track_caller]
    fn split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized;
}