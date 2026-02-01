use array_trait::{Array, length::Length};

use crate::{Bulk, InplaceBulk, IntoBulk};

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
pub unsafe trait StaticBulk: Bulk<
    MinLength = Self::Array<()>,
    MaxLength = Self::Array<()>
> + Sized
{
    type ArrayIter<U>: ExactSizeIterator<Item = U>;
    type ArrayBulk<U>: const InplaceBulk<Item = U, ItemPointee = U, IntoIter = Self::ArrayIter<U>>;
    type Array<U>: const Array<Elem = U> + Length<Elem = U> + const IntoBulk<IntoBulk = Self::ArrayBulk<U>, IntoIter = Self::ArrayIter<U>>;
}
unsafe impl<T, const N: usize> StaticBulk for T
where
    T: Bulk<
    MinLength = [(); N],
    MaxLength = [(); N]
> + Sized
{
    type ArrayIter<U> = core::array::IntoIter<U, N>;
    type ArrayBulk<U> = crate::array::IntoBulk<U, N>;
    type Array<U> = [U; N];
}