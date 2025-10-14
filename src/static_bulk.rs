use array_trait::Array;

use crate::{util::{BulkLength, Length}, Bulk, FromBulk};

#[rustc_on_unimplemented(
    message = "cannot determine the length of bulk `{Self}` at compile-time",
    label = "the bulk `{Self}` is not statically sized",
)]
pub const trait StaticBulk: ~const Bulk + BulkLength<Length: Array> + Sized
{
    type Array: Array<Elem = Self::Item> + Length + FromBulk<Self::Item, Self, Self::Array>;

    fn collect_array(self) -> Self::Array;
}