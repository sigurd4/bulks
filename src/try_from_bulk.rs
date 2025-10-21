use core::{marker::Destruct, ops::{Residual, Try}};

use crate::{util::{BulkLength, CollectLength, Length}, Bulk, IntoBulk, StaticBulk};

/// Fallible conversion from a [`Bulk`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
/// 
/// let five_fives = bulks::repeat_n(Some(5), [(); 5]);
///
/// let v = <[_; _]>::try_from_bulk(five_fives);
///
/// assert_eq!(v, Some([5, 5, 5, 5, 5]));
/// ```
///
/// Using [`Bulk::try_collect()`] to implicitly use `TryFromBulk`:
///
/// ```
/// use bulks::*;
/// 
/// let five_fives = bulks::repeat_n(Some(5), [(); 5]);
///
/// let v: Option<[_; _]> = five_fives.try_collect();
///
/// assert_eq!(v, Some([5, 5, 5, 5, 5]));
/// ```
#[rustc_on_unimplemented(
    on(
        any(Self = "&[{A}]", Self = "&[_]"),
        message = "a slice of type `{Self}` cannot be collected since we need to store the elements somewhere",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(A = "{integer}", any(Self = "&[{integral}]", Self = "&[{A}]", Self = "&[_]")),
        message = "a slice of type `{Self}` cannot be collected since we need to store the elements somewhere",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        any(Self = "[{A}]", Self = "[_]"),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(A = "{integer}", any(Self = "[{integral}]", Self = "[{A}]", Self = "[_]")),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(any(L = "[{A}]", L = "[_]"), any(Self = "[{A}; _]", Self = "[{A}; {n}]", Self = "[_; _]", Self = "[_; {n}]")),
        message = "an array of type `{Self}` cannot be collected directly from a dynamically sized bulk",
        label = "try collecting into a `Vec<{A}>`, then using `.try_into()`",
    ),
    on(
        all(L = "[{integer}]", A = "{integer}", any(Self = "[{integral}; _]", Self = "[{integral}; {n}]", Self = "[{A}; _]", Self = "[{A}; {n}]", Self = "[_; _]", Self = "[_; {n}]")),
        message = "an array of type `{Self}` cannot be collected directly from a dynamically sized bulk",
        label = "try collecting into a `Vec<{A}>`, then using `.try_into()`",
    ),
    on(
        any(L = "[{A}]", L = "[_]"),
        message = "a value of type `{Self}` cannot be collected from a dynamically sized bulk \
                of elements of type `{A}`",
        label = "value of type `{Self}` cannot be collected from `{B}` of dynamic length"
    ),
    message = "a value of type `{Self}` cannot be collected from a bulk \
               of elements of type `{A}`",
    label = "value of type `{Self}` cannot be collected from `{B}`"
)]
pub const trait TryFromBulk<A, B, L = <Self as CollectLength<A>>::Length>: Sized
where
    B: BulkLength<Item: Try<Output = A, Residual: Residual<Self>>>,
    L: Length<Elem = A> + ?Sized
{
    /// Fallably creates a value from a bulk.
    ///
    /// See the [crate-level documentation](crate) for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let five_fives = bulks::repeat_n(Some(5), [(); 5]);
    ///
    /// let v = <[_; _]>::try_from_bulk(five_fives);
    ///
    /// assert_eq!(v, Some([5, 5, 5, 5, 5]));
    /// ```
    fn try_from_bulk<I>(bulk: I) -> <<B::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<Item = B::Item, IntoBulk = B>;
}

impl<A, B, T, R> TryFromBulk<A, B, [A]> for T
where
    T: FromIterator<A>,
    B: Bulk<Item = R>,
    R: Try<Output = A, Residual: Residual<Self>>
{
    fn try_from_bulk<I>(bulk: I) -> <R::Residual as Residual<Self>>::TryType
    where
        I: IntoBulk<Item = R, IntoBulk = B>
    {
        bulk.into_iter().try_collect()
    }
}
impl<A, B, R, const N: usize> const TryFromBulk<A, B, [A; N]> for [A; N]
where
    A: ~const Destruct,
    B: ~const Bulk + StaticBulk<Item = R, Array<A> = [A; N]>,
    R: ~const Try<Output = A, Residual: Residual<(), TryType: ~const Try> + Residual<Self, TryType: ~const Try> + ~const Destruct> + ~const Destruct
{
    fn try_from_bulk<I>(bulk: I) -> <R::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<Item = R, IntoBulk = B>
    {
        bulk.into_bulk().try_collect_array()
    }
}