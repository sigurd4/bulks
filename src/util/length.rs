use core::ptr::Pointee;

use array_trait::AsSlice;

pub trait Length: AsSlice
{
    fn len_metadata(n: <Self as Pointee>::Metadata) -> usize;
}
impl<T> Length for [T]
{
    fn len_metadata(n: <Self as Pointee>::Metadata) -> usize
    {
        n
    }
}
impl<T, const N: usize> Length for [T; N]
{
    fn len_metadata((): <Self as Pointee>::Metadata) -> usize
    {
        N
    }
}