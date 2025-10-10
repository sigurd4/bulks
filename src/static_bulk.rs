use array_trait::Array;

use crate::{util::Length, Bulk};

#[rustc_on_unimplemented(
    message = "cannot determine the length of bulk `{Self}` at compile-time",
    label = "the bulk `{Self}` is not statically sized",
)]
pub trait StaticBulk: Bulk
{
    type Array: Array<Elem = Self::Item> + Length;

    fn collect_array(self) -> Self::Array;
}