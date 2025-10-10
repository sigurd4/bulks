use core::{range::IterRangeFrom, ops::RangeFrom};

#[rustc_on_unimplemented(
    message = "iterator `{Self}` is considered finite",
    label = "iterator `{Self}` is considered finite",
)]
pub unsafe trait InfiniteIterator: Iterator
{

}

unsafe impl<Idx> InfiniteIterator for RangeFrom<Idx>
where
    Self: Iterator {}
unsafe impl<Idx> InfiniteIterator for IterRangeFrom<Idx>
where
    Self: Iterator {}