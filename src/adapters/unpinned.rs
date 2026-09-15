use core::{iter, marker::Destruct, ops::Deref, pin::Pin};

use crate::Bulk;

pub struct Unpinned<B, T>
where
    B: Bulk<Item = Pin<T>>,
    T: Deref
{
    bulk: B
}

impl<B, T> Unpinned<B, T>
where
    B: Bulk<Item = Pin<T>>,
    T: Deref
{
    pub(crate) const fn new(bulk: B) -> Self
    where
        T::Target: Unpin
    {
        let _ = |x: Pin<T>| Pin::into_inner(x);

        unsafe {Self::new_unchecked(bulk)}
    }

    pub(crate) const unsafe fn new_unchecked(bulk: B) -> Self
    {
        Self { bulk }
    }
}

const impl<B, T> IntoIterator for Unpinned<B, T>
where
    B: Bulk<Item = Pin<T>, IntoIter: [const] Iterator> + [const] IntoIterator,
    T: Deref
{
    type IntoIter = iter::Map<B::IntoIter, impl Fn(Pin<T>) -> T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter
    {
        self.bulk.into_iter().map(Functor)
    }
}
const impl<B, T> Bulk for Unpinned<B, T>
where
    B: [const] Bulk<Item = Pin<T>>,
    T: Deref + [const] Destruct
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
        F: [const] FnMut(Self::Item) -> R + [const] core::marker::Destruct,
        R: [const] core::ops::Try<Output = (), Residual: [const] core::marker::Destruct>
    {
        self.bulk.map(Functor).try_for_each(f)
    }
}

struct Functor;
const impl<T> FnOnce<(Pin<T>,)> for Functor
where
    T: Deref
{
    type Output = T;

    extern "rust-call" fn call_once(self, args: (Pin<T>,)) -> Self::Output
    {
        self.call(args)
    }
}
const impl<T> FnMut<(Pin<T>,)> for Functor
where
    T: Deref
{
    extern "rust-call" fn call_mut(&mut self, args: (Pin<T>,)) -> Self::Output
    {
        self.call(args)
    }
}
const impl<T> Fn<(Pin<T>,)> for Functor
where
    T: Deref
{
    extern "rust-call" fn call(&self, (x,): (Pin<T>,)) -> Self::Output
    {
        unsafe { Pin::into_inner_unchecked(x) }
    }
}
