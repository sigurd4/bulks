use core::ops::Residual;
use core::ops::Try;

use array_trait::Array;
use array_trait::length;

use crate::Length;
use crate::util;
use crate::{Bulk, FromBulk, IntoBulk, Map, StaticBulk, TryFromBulk, util::{BulkLength}};

pub const trait NearestFrom<B: BulkLength + ?Sized, L: length::Length<Elem = B::Item> + ?Sized = <B as BulkLength>::Length> = ~const FromBulk<B::Item, B, L> + IntoBulk<Item = B::Item>;

pub trait Nearest<L = <Self as BulkLength>::Length>: Bulk
where
    L: length::Length<Elem = Self::Item> + ?Sized
{
    type Nearest: NearestFrom<Self>;
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
    A: Array<Elem = T> + length::Length + util::CollectLength<T, Length = A> + IntoBulk<Item = T> + FromBulk<T, Self, A>
{
    type Nearest = A;
}

pub const trait TryNearestFrom<B: BulkLength<Item: Try<Residual: Residual<Self>>> + ?Sized, L: length::Length<Elem = B::Item> + ?Sized = Length<B>> = ~const TryFromBulk<<B::Item as Try>::Output, B, length::Mapped<Length<B>, <B::Item as Try>::Output>> + IntoBulk<Item = <B::Item as Try>::Output>;

pub trait TryNearest: Bulk<Item: Try<Residual: Residual<<Self as TryNearest>::TryNearest>>>
{
    type TryNearest: TryNearestFrom<Self>;
}
impl<R, T, B> TryNearest for B
where
    B: Bulk<Item = R>,
    Map<Self, Unwrapper>: Bulk<Item = T, CollectNearest: TryNearestFrom<Self>> + Nearest,
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