use core::{alloc::Allocator, ops::Try};

use alloc::vec::Vec;
use array_trait::length::{self, LengthValue};

use crate::{AsBulk, Bulk, DoubleEndedBulk, InplaceBulk, IntoBulk, RandomAccessBulk, SplitBulk, slice};

pub mod vec
{
    use core::alloc::Allocator;

    use alloc::{vec::Vec, alloc::Global};

    pub struct IntoBulk<T, A = Global>
    where
        A: Allocator
    {
        pub(super) inner: Vec<T, A>
    }
}

impl<T, A> IntoIterator for vec::IntoBulk<T, A>
where
    A: Allocator
{
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T, A>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.inner.into_iter()
    }
}
impl<T, A> IntoBulk for Vec<T, A>
where
    A: Allocator
{
    type IntoBulk = vec::IntoBulk<T, A>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        vec::IntoBulk {
            inner: self
        }
    }
}
impl<'a, T, A> IntoBulk for &'a Vec<T, A>
where
    A: Allocator
{
    type IntoBulk = slice::Bulk<'a, T>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        self.as_slice().bulk()
    }
}
impl<'a, T, A> IntoBulk for &'a mut Vec<T, A>
where
    A: Allocator
{
    type IntoBulk = slice::BulkMut<'a, T>;
    
    fn into_bulk(self) -> Self::IntoBulk
    {
        self.as_mut_slice().bulk_mut()
    }
}
impl<T, A> Bulk for vec::IntoBulk<T, A>
where
    A: Allocator
{
    fn len(&self) -> usize
    {
        self.inner.len()
    }
    fn is_empty(&self) -> bool
    {
        self.inner.is_empty()
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.into_iter().for_each(f);
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().try_for_each(f)
    }

    fn get<'a, L>(&'a self, i: L) -> Option<&'a <Self as RandomAccessBulk>::ItemPointee>
    where
        Self: 'a,
        L: LengthValue
    {
        self.inner.get(length::value::len(i))
    }
    fn get_mut<'a, L>(&'a mut self, i: L) -> Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>
    where
        Self: 'a,
        L: LengthValue
    {
        self.inner.get_mut(length::value::len(i))
    }
}

impl<T, A> DoubleEndedBulk for vec::IntoBulk<T, A>
where
    A: Allocator
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item)
    {
        self.into_iter().rev().for_each(f);
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Try<Output = ()>
    {
        self.into_iter().rev().try_for_each(f)
    }
}

impl<T, A, L> SplitBulk<L> for vec::IntoBulk<T, A>
where
    A: Allocator + Clone,
    L: LengthValue
{
    type Left = Self;
    type Right = Self;

    fn split_at(mut left: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let Self { inner } = &mut left;

        let right = inner.split_off(length::value::len(n))
            .into_bulk();

        (left, right)
    }
}

impl<T, A> RandomAccessBulk for vec::IntoBulk<T, A>
where
    A: Allocator
{
    type ItemPointee = T;
    type EachRef<'a> = slice::Bulk<'a, T>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(Self { inner }: &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        inner.bulk()
    }
}
impl<T, A> InplaceBulk for vec::IntoBulk<T, A>
where
    A: Allocator
{
    type EachMut<'a> = slice::BulkMut<'a, T>
    where
        Self::ItemPointee: 'a,
        Self: 'a;
    
    fn each_mut<'a>(Self { inner }: &'a mut Self) -> Self::EachMut<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        inner.bulk_mut()
    }
}