use core::marker::Destruct;
use core::ops::Try;

use crate::{Bulk, IntoBulk};
use array_trait::{AsSlice, length::{self, Length}};

use crate::util::ArrayBuffer;

#[cfg(not(feature = "alloc"))]
use array_trait::Array;

#[cfg(not(feature = "alloc"))]
pub trait BufferableBulk = crate::StaticBulk;
#[cfg(feature = "alloc")]
pub trait BufferableBulk = Bulk;

#[cfg(not(feature = "alloc"))]
pub trait BufferLength = Length<Elem = ()> + Array;
#[cfg(feature = "alloc")]
pub trait BufferLength = Length<Elem = ()>;

pub struct Buffer<T, L>
where
    L: BufferLength + ?Sized
{
    buffer: <L as BufferLengthSpec>::Buffer<T>
}
impl<T, L> Buffer<T, L>
where
    L: BufferLength + ?Sized
{
    #[allow(private_bounds)]
    pub const fn new(capacity: L::Value) -> Self
    where
        <L as BufferLengthSpec>::Buffer<T>: [const] BufferSpec<T>
    {
        Self {
            buffer: BufferSpec::new(length::value::len(capacity))
        }
    }

    #[allow(private_bounds)]
    pub const fn push(&mut self, value: T)
    where
        <L as BufferLengthSpec>::Buffer<T>: [const] BufferSpec<T>
    {
        self.buffer.push(value)
    }

    #[allow(private_bounds)]
    pub const fn push_mut(&mut self, value: T) -> &mut T
    where
        <L as BufferLengthSpec>::Buffer<T>: [const] BufferSpec<T>
    {
        self.buffer.push_mut(value)
    }

    #[allow(private_bounds)]
    pub const fn len(&self) -> usize
    where
        <L as BufferLengthSpec>::Buffer<T>: [const] BufferSpec<T>
    {
        self.buffer.len()
    }
}

const impl<T, L> AsSlice for Buffer<T, L>
where
    L: BufferLength + ?Sized
{
    type Elem = T;

    fn as_slice(&self) -> &[Self::Elem]
    {
        self.buffer.as_slice()
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Elem]
    {
        self.buffer.as_mut_slice()
    }
}

const impl<T, L> IntoIterator for Buffer<T, L>
where
    L: BufferLength + ?Sized,
    <L as BufferLengthSpec>::Buffer<T>: [const] IntoIterator
{
    type IntoIter = BufferIter<T, L>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        BufferIter { iter: self.buffer.into_iter() }
    }
}
const impl<T, L> IntoBulk for Buffer<T, L>
where
    L: BufferLength + ?Sized,
    <L as BufferLengthSpec>::Buffer<T>: [const] IntoBulk
{
    type IntoBulk = BufferBulk<T, L>;

    fn into_bulk(self) -> Self::IntoBulk
    {
        BufferBulk { bulk: self.buffer.into_bulk() }
    }
}
impl<'a, T, L> IntoIterator for &'a Buffer<T, L>
where
    L: BufferLength + ?Sized
{
    type IntoIter = core::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter
    {
        self.buffer.as_slice().iter()
    }
}
impl<'a, T, L> IntoIterator for &'a mut Buffer<T, L>
where
    L: BufferLength + ?Sized
{
    type IntoIter = core::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter
    {
        self.buffer.as_mut_slice().iter_mut()
    }
}

pub struct BufferIter<T, L>
where
    L: BufferLength + ?Sized
{
    iter: <<L as BufferLengthSpec>::Buffer<T> as IntoIterator>::IntoIter
}
const impl<T, L> Iterator for BufferIter<T, L>
where
    L: BufferLength + ?Sized,
    <<L as BufferLengthSpec>::Buffer<T> as IntoIterator>::IntoIter: [const] Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iter.next()
    }
}
impl<T, L> ExactSizeIterator for BufferIter<T, L>
where
    L: BufferLength + ?Sized
{
    fn len(&self) -> usize
    {
        self.iter.len()
    }
}

pub struct BufferBulk<T, L>
where
    L: BufferLength + ?Sized
{
    bulk: <<L as BufferLengthSpec>::Buffer<T> as IntoBulk>::IntoBulk
}
const impl<T, L> IntoIterator for BufferBulk<T, L>
where
    L: BufferLength + ?Sized,
    <L as BufferLengthSpec>::Buffer<T>: [const] IntoBulk,
    <<L as BufferLengthSpec>::Buffer<T> as IntoBulk>::IntoBulk: [const] IntoIterator<IntoIter = <<L as BufferLengthSpec>::Buffer<T> as IntoIterator>::IntoIter>
{
    type IntoIter = BufferIter<T, L>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        BufferIter { iter: self.bulk.into_iter() }
    }
}
const impl<T, L> Bulk for BufferBulk<T, L>
where
    L: BufferLength + ?Sized,
    <<L as BufferLengthSpec>::Buffer<T> as IntoBulk>::IntoBulk: [const] Bulk
{
    type MaxLength = L;
    type MinLength = [(); 0];

    fn len(&self) -> usize
    {
        self.bulk.len()
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: [const] FnMut(Self::Item) + [const] Destruct
    {
        self.bulk.for_each(f);
    }

    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: [const] Destruct,
        F: [const] FnMut(Self::Item) -> R + [const] Destruct,
        R: [const] Try<Output = ()>
    {
        self.bulk.try_for_each(f)
    }
}

const trait BufferSpec<T>: IntoBulk<Item = T> + const AsSlice<Elem = T>
{
    fn new(capacity: usize) -> Self;

    fn push(&mut self, value: T);
    fn push_mut(&mut self, value: T) -> &mut T;

    fn len(&self) -> usize;
}
#[cfg(feature = "alloc")]
impl<T> BufferSpec<T> for alloc::vec::Vec<T>
{
    fn new(capacity: usize) -> Self
    {
        alloc::vec::Vec::with_capacity(capacity)
    }

    fn push(&mut self, value: T)
    {
        self.push(value);
    }

    fn push_mut(&mut self, value: T) -> &mut T
    {
        self.push_mut(value)
    }

    fn len(&self) -> usize
    {
        self.len()
    }
}
const impl<T, const N: usize> BufferSpec<T> for ArrayBuffer<T, N, false>
{
    fn new(_capacity: usize) -> Self
    {
        Self::new()
    }

    fn push(&mut self, value: T)
    {
        self.push(value);
    }

    fn push_mut(&mut self, value: T) -> &mut T
    {
        self.push_mut(value)
    }

    fn len(&self) -> usize
    {
        self.len()
    }
}

trait BufferLengthSpec: Length<Elem = ()>
{
    type Buffer<T>: BufferSpec<T>;
}
#[cfg(not(feature = "alloc"))]
impl<L> BufferLengthSpec for L
where
    L: BufferLength + ?Sized
{
    default type Buffer<T> = ArrayBuffer<T, 0, false>;
}
#[cfg(feature = "alloc")]
impl<L> BufferLengthSpec for L
where
    L: BufferLength + ?Sized
{
    default type Buffer<T> = alloc::vec::Vec<T>;
}
impl<const N: usize> BufferLengthSpec for [(); N]
{
    type Buffer<T> = ArrayBuffer<T, N, false>;
}
