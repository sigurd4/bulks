use core::{marker::Destruct, ops::Try};

use crate::Bulk;

pub const trait DoubleEndedBulk: Bulk<IntoIter: DoubleEndedIterator>
{
    /// Calls a closure on each element of a bulk in reverse.
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct;

    /// A bulk method that applies a fallible function to each item in the
    /// bulk in reverse, stopping at the first error and returning that error.
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>;
}