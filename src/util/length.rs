use core::ptr::Pointee;

use array_trait::AsSlice;

#[rustc_on_unimplemented(
    message = "`{Self}` is not a valid bulk length",
    label = "The only valid lengths are `[_]` or `[_; _]`",
)]
pub const trait Length: AsSlice
{
    type LengthSpec: const LengthSpec<Length<Self::Elem> = Self, Metadata = <Self as Pointee>::Metadata>;

    fn len_metadata(n: <Self as Pointee>::Metadata) -> usize;
}
impl<T> const Length for [T]
{
    type LengthSpec = usize;

    fn len_metadata(n: <Self as Pointee>::Metadata) -> usize
    {
        n
    }
}
impl<T, const N: usize> const Length for [T; N]
{
    type LengthSpec = [(); N];

    fn len_metadata((): <Self as Pointee>::Metadata) -> usize
    {
        N
    }
}

pub const trait LengthSpec: Copy
{
    type Length<T>: const Length<Elem = T, LengthSpec = Self> + Pointee<Metadata = Self::Metadata> + ?Sized;
    type Metadata: Copy;
    
    fn into_metadata(self) -> Self::Metadata;
    fn len_metadata(self) -> usize;
}
impl const LengthSpec for usize
{
    type Length<T> = [T];
    type Metadata = usize;

    fn into_metadata(self) -> Self::Metadata
    {
        self
    }
    fn len_metadata(self) -> usize
    {
        self
    }
}
impl<const N: usize> const LengthSpec for [(); N]
{
    type Length<T> = [T; N];
    type Metadata = ();

    fn into_metadata(self) -> Self::Metadata
    {
        
    }
    fn len_metadata(self) -> usize
    {
        N
    }
}