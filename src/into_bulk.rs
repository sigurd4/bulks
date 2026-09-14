use core::ptr::Thin;

use crate::Bulk;

pub const trait AsBulk<'a>
{
    type AsBulk: Bulk + 'a;

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
    fn bulk(&'a self) -> Self::AsBulk;
}
pub const trait AsBulkMut<'a>
{
    type AsBulkMut: Bulk + 'a;

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
    fn bulk_mut(&'a mut self) -> Self::AsBulkMut;
}

const impl<'a, T> AsBulk<'a> for T
where
    T: ?Sized + 'a,
    &'a T: ~const IntoBulk
{
    type AsBulk = <&'a T as IntoBulk>::IntoBulk;

    fn bulk(&'a self) -> Self::AsBulk {
        self.into_bulk()
    }
}
const impl<'a, T> AsBulkMut<'a> for T
where
    T: ?Sized + 'a,
    &'a mut T: ~const IntoBulk
{
    type AsBulkMut = <&'a mut T as IntoBulk>::IntoBulk;

    fn bulk_mut(&'a mut self) -> Self::AsBulkMut {
        self.into_bulk()
    }
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

const impl<T> IntoBulk for T
where 
    Self: ~const Bulk
{
    type IntoBulk = Self;

    fn into_bulk(self) -> Self::IntoBulk
    {
        self
    }
}
