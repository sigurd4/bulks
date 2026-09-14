use core::ptr::Thin;

use crate::Bulk;

pub const trait AsBulk<'a>: 'a
{
    type Elem: 'a;
    type AsBulk: Bulk<Item = &'a Self::Elem> + 'a;

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
pub const trait AsBulkMut<'a>: AsBulk<'a>
{
    type AsBulkMut: Bulk<Item = &'a mut Self::Elem> + 'a;

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

const impl<'a, B, T> AsBulk<'a> for B
where
    B: ?Sized + 'a,
    T: 'a,
    &'a B: [const] IntoBulk<Item = &'a T>
{
    type AsBulk = <&'a B as IntoBulk>::IntoBulk;
    type Elem = T;

    fn bulk(&'a self) -> Self::AsBulk
    {
        self.into_bulk()
    }
}
const impl<'a, B, T> AsBulkMut<'a> for B
where
    B: ?Sized + 'a,
    T: 'a,
    &'a B: [const] IntoBulk<Item = &'a T>,
    &'a mut B: [const] IntoBulk<Item = &'a mut T>
{
    type AsBulkMut = <&'a mut B as IntoBulk>::IntoBulk;

    fn bulk_mut(&'a mut self) -> Self::AsBulkMut
    {
        self.into_bulk()
    }
}

pub const trait IntoBulk: IntoIterator<Item: Thin, IntoIter: ExactSizeIterator>
{
    /// Which kind of bulk are we turning this into?
    type IntoBulk: [const] Bulk<Item = Self::Item, IntoIter = Self::IntoIter>;

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
    Self: [const] Bulk
{
    type IntoBulk = Self;

    fn into_bulk(self) -> Self::IntoBulk
    {
        self
    }
}
