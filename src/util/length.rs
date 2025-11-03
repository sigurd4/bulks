use core::{marker::Destruct, ptr::Pointee};

use array_trait::AsSlice;

use crate::util::Same;

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

pub const trait LengthSpec: Copy + const Destruct
{
    type Length<T>: const Length<Elem = T, LengthSpec = Self> + Pointee<Metadata = Self::Metadata> + ?Sized;
    type Metadata: Copy;
    
    fn from_metadata(n: Self::Metadata) -> Self;
    fn into_metadata(self) -> Self::Metadata;
    fn len_metadata(self) -> usize;
}
impl const LengthSpec for usize
{
    type Length<T> = [T];
    type Metadata = usize;

    fn from_metadata(n: Self::Metadata) -> Self
    {
        n
    }
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

    fn from_metadata((): Self::Metadata) -> Self
    {
        [(); N]
    }
    fn into_metadata(self) -> Self::Metadata
    {
        
    }
    fn len_metadata(self) -> usize
    {
        N
    }
}

pub const trait LengthMul<const N: usize>: LengthSpec
{
    type LengthMul: LengthSpec;

    fn len_mul(self) -> Self::LengthMul;
}
impl<L, const N: usize> const LengthMul<N> for L
where
    L: ~const LengthSpec
{
    default type LengthMul = usize;

    default fn len_mul(self) -> Self::LengthMul
    {
        (self.len_metadata()*N).same().ok().unwrap()
    }
}
impl<const M: usize, const N: usize> const LengthMul<N> for [(); M]
where
    [(); M*N]:
{
    type LengthMul = [(); M*N];

    fn len_mul(self) -> Self::LengthMul
    {
        [(); M*N]
    }
}

pub const trait LengthDiv<const N: usize>: LengthSpec
{
    type LengthDiv: LengthSpec;

    fn len_div(self) -> Self::LengthDiv;
}
impl<L, const N: usize> const LengthDiv<N> for L
where
    L: ~const LengthSpec
{
    default type LengthDiv = usize;

    default fn len_div(self) -> Self::LengthDiv
    {
        (self.len_metadata()/N).same().ok().unwrap()
    }
}
impl<const M: usize, const N: usize> const LengthDiv<N> for [(); M]
where
    [(); M/N]:
{
    type LengthDiv = [(); M/N];

    fn len_div(self) -> Self::LengthDiv
    {
        [(); M/N]
    }
}