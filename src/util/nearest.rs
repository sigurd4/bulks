use core::ops::Residual;
use core::ops::Try;

use array_trait::Array;

use crate::{Bulk, FromBulk, IntoBulk, Map, StaticBulk, TryFromBulk, util::{BulkLength, Length, LengthSpec}};

pub trait Nearest<L = <Self as BulkLength>::Length>: Bulk
where
    L: Length<Elem = Self::Item> + ?Sized
{
    type Nearest: FromBulk<Self::Item, Self, L> + IntoBulk<Item = Self::Item>;
}
#[cfg(feature = "alloc")]
impl<T, B> Nearest<[T]> for B
where
    B: BulkLength<Item = T, Length = [T]>
{
    type Nearest = alloc::vec::Vec<T>;
}
impl<T, B, A> Nearest<A> for B
where
    B: StaticBulk<Item = T, Array<T> = A> + BulkLength<Length = A>,
    A: Array<Elem = T> + Length + IntoBulk<Item = T> + FromBulk<T, Self, A>
{
    type Nearest = A;
}

pub trait TryNearest: Bulk<Item: Try<Residual: Residual<<Self as TryNearest>::TryNearest>>>
{
    type TryNearest: TryFromBulk<<Self::Item as Try>::Output, Self, <<<Self as BulkLength>::Length as Length>::LengthSpec as LengthSpec>::Length<<<Self as IntoIterator>::Item as Try>::Output>> + IntoBulk<Item = <Self::Item as Try>::Output>;
}
impl<R, T, B> TryNearest for B
where
    B: Bulk<Item = R>,
    Map<Self, Unwrapper>: Bulk<Item = T, CollectNearest: TryFromBulk<<Self::Item as Try>::Output, Self, <<<Self as BulkLength>::Length as Length>::LengthSpec as LengthSpec>::Length<T>> + IntoBulk<Item = T>>
    + Nearest,
    R: Try<Output = T, Residual: Residual<<Map<Self, Unwrapper> as Bulk>::CollectNearest>>
{
    type TryNearest = <Map<Self, Unwrapper> as Bulk>::CollectNearest;
}

pub struct Unwrapper;

impl<R, T> FnOnce<(R,)> for Unwrapper
where
    R: Try<Output = T>
{
    type Output = T;

    extern "rust-call" fn call_once(self, args: (R,)) -> Self::Output
    {
        self.call(args)
    }
}
impl<R, T> FnMut<(R,)> for Unwrapper
where
    R: Try<Output = T>
{
    extern "rust-call" fn call_mut(&mut self, args: (R,)) -> Self::Output
    {
        self.call(args)
    }
}
impl<R, T> Fn<(R,)> for Unwrapper
where
    R: Try<Output = T>
{
    extern "rust-call" fn call(&self, (r,): (R,)) -> Self::Output
    {
        r.branch().continue_value().unwrap()
    }
}