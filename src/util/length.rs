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
    type Metadata: Copy + const Default;
    
    fn or_len_metadata(n: usize) -> Self;
    fn from_metadata(n: Self::Metadata) -> Self;
    fn into_metadata(self) -> Self::Metadata;
    fn len_metadata(self) -> usize;
}
impl const LengthSpec for usize
{
    type Length<T> = [T];
    type Metadata = usize;

    fn or_len_metadata(n: usize) -> Self
    {
        n
    }
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

    fn or_len_metadata(_: usize) -> Self
    {
        [(); N]
    }
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
pub const trait LengthMin<R>: LengthSpec
where
    R: LengthSpec
{
    type LengthMin: LengthSpec;

    fn len_min(self, other: R) -> Self::LengthMin;
}
impl<L, R> const LengthMin<R> for L
where
    L: ~const LengthSpec,
    R: ~const LengthSpec
{
    default type LengthMin = usize;

    default fn len_min(self, other: R) -> Self::LengthMin
    {
        self.len_metadata().min(other.len_metadata()).same().ok().unwrap()
    }
}
impl<const M: usize, const N: usize> const LengthMin<[(); N]> for [(); M]
where
    [(); M.min(N)]:
{
    type LengthMin = [(); M.min(N)];

    fn len_min(self, _: [(); N]) -> Self::LengthMin
    {
        [(); M.min(N)]
    }
}

pub const trait LengthAdd<R>: LengthSpec
where
    R: LengthSpec
{
    type LengthAdd: LengthSpec;

    fn len_add(self, other: R) -> Self::LengthAdd;
}
impl<L, R> const LengthAdd<R> for L
where
    L: ~const LengthSpec,
    R: ~const LengthSpec
{
    default type LengthAdd = usize;

    default fn len_add(self, other: R) -> Self::LengthAdd
    {
        self.len_metadata().saturating_add(other.len_metadata()).same().ok().unwrap()
    }
}
impl<const M: usize, const N: usize> const LengthAdd<[(); N]> for [(); M]
where
    [(); M.saturating_add(N)]:
{
    type LengthAdd = [(); M.saturating_add(N)];

    fn len_add(self, _: [(); N]) -> Self::LengthAdd
    {
        [(); M.saturating_add(N)]
    }
}

pub const trait LengthSub<R>: LengthSpec
where
    R: LengthSpec
{
    type LengthSub: LengthSpec;

    fn len_sub(self, other: R) -> Self::LengthSub;
}
impl<L, R> const LengthSub<R> for L
where
    L: ~const LengthSpec,
    R: ~const LengthSpec
{
    default type LengthSub = usize;

    default fn len_sub(self, other: R) -> Self::LengthSub
    {
        self.len_metadata().saturating_sub(other.len_metadata()).same().ok().unwrap()
    }
}
impl<const M: usize, const N: usize> const LengthSub<[(); N]> for [(); M]
where
    [(); M.saturating_sub(N)]:
{
    type LengthSub = [(); M.saturating_sub(N)];

    fn len_sub(self, _: [(); N]) -> Self::LengthSub
    {
        [(); M.saturating_sub(N)]
    }
}

pub const trait LengthMul<R>: LengthSpec
where
    R: LengthSpec
{
    type LengthMul: LengthSpec;

    fn len_mul(self, other: R) -> Self::LengthMul;
}
impl<L, R> const LengthMul<R> for L
where
    L: ~const LengthSpec,
    R: ~const LengthSpec
{
    default type LengthMul = usize;

    default fn len_mul(self, other: R) -> Self::LengthMul
    {
        self.len_metadata().saturating_mul(other.len_metadata()).same().ok().unwrap()
    }
}
impl<const M: usize, const N: usize> const LengthMul<[(); N]> for [(); M]
where
    [(); M.saturating_mul(N)]:
{
    type LengthMul = [(); M.saturating_mul(N)];

    fn len_mul(self, _: [(); N]) -> Self::LengthMul
    {
        [(); M.saturating_mul(N)]
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