use core::ptr::Pointee;

use array_trait::AsSlice;

#[rustc_on_unimplemented(
    message = "`{Self}` is not a valid bulk length",
    label = "The only valid lengths are `[_]` or `[_; _]`",
)]
pub const trait Length: AsSlice
{
    fn len_metadata(n: <Self as Pointee>::Metadata) -> usize;
}
impl<T> const Length for [T]
{
    fn len_metadata(n: <Self as Pointee>::Metadata) -> usize
    {
        n
    }
}
impl<T, const N: usize> const Length for [T; N]
{
    fn len_metadata((): <Self as Pointee>::Metadata) -> usize
    {
        N
    }
}