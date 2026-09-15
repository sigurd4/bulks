use core::{iter, marker::Destruct, ops::Deref, pin::Pin};

use array_trait::length::LengthValue;

use crate::{Bulk, DoubleEndedBulk, SplitBulk};

pub struct Pinned<B>
where
    B: Bulk<Item: Deref>
{
    bulk: B
}

impl<B> Pinned<B>
where
    B: Bulk<Item: Deref>
{
    pub(crate) const fn new(bulk: B) -> Self
    where
        <B::Item as Deref>::Target: Unpin
    {
        let _ = |x: B::Item| Pin::new(x);

        unsafe { Self::new_unchecked(bulk) }
    }

    pub(crate) const unsafe fn new_unchecked(bulk: B) -> Self
    {
        Self { bulk }
    }
}

const impl<B> IntoIterator for Pinned<B>
where
    B: Bulk<Item: Deref, IntoIter: [const] Iterator> + [const] IntoIterator
{
    type IntoIter = iter::Map<B::IntoIter, impl Fn(B::Item) -> Pin<B::Item>>;
    type Item = Pin<B::Item>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.bulk.into_iter().map(Functor)
    }
}
const impl<B> Bulk for Pinned<B>
where
    B: [const] Bulk<Item: Deref + [const] Destruct>
{
    type MaxLength = B::MaxLength;
    type MinLength = B::MinLength;

    fn len(&self) -> usize
    {
        self.bulk.len()
    }

    fn length(&self) -> array_trait::length::Value<crate::BulkLength<Self>>
    {
        self.bulk.length()
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: [const] FnMut(Self::Item) + [const] core::marker::Destruct
    {
        self.bulk.map(Functor).for_each(f);
    }

    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: [const] core::marker::Destruct,
        F: [const] FnMut(Self::Item) -> R + [const] core::marker::Destruct,
        R: [const] core::ops::Try<Output = ()>
    {
        self.bulk.map(Functor).try_for_each(f)
    }
}
const impl<B> DoubleEndedBulk for Pinned<B>
where
    B: [const] DoubleEndedBulk<Item: Deref + [const] Destruct>
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: [const] FnMut(Self::Item) + [const] Destruct
    {
        self.bulk.map(Functor).rev_for_each(f)
    }

    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: [const] Destruct,
        F: [const] FnMut(Self::Item) -> R + [const] Destruct,
        R: [const] core::ops::Try<Output = ()>
    {
        self.bulk.map(Functor).try_rev_for_each(f)
    }
}
const impl<B, L> SplitBulk<L> for Pinned<B>
where
    B: [const] SplitBulk<L, Item: Deref + [const] Destruct>,
    L: LengthValue
{
    type Left = Pinned<B::Left>;
    type Right = Pinned<B::Right>;

    fn split_at(bulk: Self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized
    {
        let (left, right) = bulk.bulk.split_at(n);

        unsafe { (Pinned::new_unchecked(left), Pinned::new_unchecked(right)) }
    }
}

struct Functor;
const impl<T> FnOnce<(T,)> for Functor
where
    T: Deref
{
    type Output = Pin<T>;

    extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output
    {
        self.call(args)
    }
}
const impl<T> FnMut<(T,)> for Functor
where
    T: Deref
{
    extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output
    {
        self.call(args)
    }
}
const impl<T> Fn<(T,)> for Functor
where
    T: Deref
{
    extern "rust-call" fn call(&self, (x,): (T,)) -> Self::Output
    {
        unsafe { Pin::new_unchecked(x) }
    }
}
