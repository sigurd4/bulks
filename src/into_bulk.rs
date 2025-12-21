use core::ptr::Thin;

use crate::Bulk;

pub const trait AsBulk
{
    /// Creates a bulk from a reference.
    ///
    /// See the [crate documentation](crate) for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let v = [1, 2, 3];
    /// let bulk = v.bulk();
    /// let u: [_; _] = bulk.collect();
    ///
    /// assert_eq!(u, [&1, &2, &3]);
    /// ```
    fn bulk<'a>(&'a self) -> <&'a Self as IntoBulk>::IntoBulk
    where
        &'a Self: ~const IntoBulk
    {
        self.into_bulk()
    }

    /// Creates a bulk from a mutable reference.
    ///
    /// See the [crate documentation](crate) for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let mut v = [1, 2, 3];
    /// let bulk = v.bulk_mut();
    /// let u: [_; _] = bulk.map(|v| core::mem::replace(v, *v + 1))
    ///     .collect();
    ///
    /// assert_eq!(v, [2, 3, 4]);
    /// assert_eq!(u, [1, 2, 3]);
    /// ```
    fn bulk_mut<'a>(&'a mut self) -> <&'a mut Self as IntoBulk>::IntoBulk
    where
        &'a mut Self: ~const IntoBulk
    {
        self.into_bulk()
    }
}

impl<T> const AsBulk for T
where
    T: ?Sized
{
    
}

pub const trait IntoBulk: IntoIterator<Item: Thin, IntoIter: ExactSizeIterator>
{
    /// Which kind of bulk are we turning this into?
    type IntoBulk: ~const Bulk<Item = Self::Item, IntoIter = Self::IntoIter>;

    /// Creates a bulk from a value.
    ///
    /// See the [crate documentation](crate) for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let v = [1, 2, 3];
    /// let mut bulk = v.into_bulk();
    /// let u: [_; _] = bulk.collect();
    ///
    /// assert_eq!(u, [1, 2, 3]);
    /// ```
    fn into_bulk(self) -> Self::IntoBulk;
}

impl<T> const IntoBulk for T
where 
    Self: ~const Bulk
{
    type IntoBulk = Self;

    fn into_bulk(self) -> Self::IntoBulk
    {
        self
    }
}