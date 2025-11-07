use array_trait::{Array, length};

use crate::{Bulk, IntoBulk, util::BulkLength};

/// A trait for bulks whose length can be determined at compile-time.
/// 
/// # Safety
/// 
/// You must guarantee that the bulk will always yield an exact amount of elements, predetermined at compile-time.
/// The bulk's length must always be the same as the length of [`Self::Array`](StaticBulk::Array).
#[rustc_on_unimplemented(
    message = "cannot determine the length of bulk `{Self}` at compile-time",
    label = "the bulk `{Self}` is not statically sized",
)]
pub unsafe trait StaticBulk: Bulk<MinLength<<Self as IntoIterator>::Item> = Self::Array<<Self as IntoIterator>::Item>, MaxLength<<Self as IntoIterator>::Item> = Self::Array<<Self as IntoIterator>::Item>> + BulkLength<Length: Array> + Sized
{
    type Array<U>: const Array<Elem = U> + length::Length<Elem = U> + IntoBulk<Item = U>;
}