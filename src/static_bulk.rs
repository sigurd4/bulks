use core::{marker::Destruct, mem::MaybeUninit, ops::Range};

use array_trait::Array;

use crate::{util::{BulkLength, Length}, Bulk};

pub(crate) struct Guard<'a, T> {
    /// The array to be initialized.
    pub array_mut: &'a mut [MaybeUninit<T>],
    /// The number of items that have been initialized so far.
    pub initialized: Range<usize>,
}
impl<T> Guard<'_, T> {
    /// Adds an item to the array and updates the initialized item counter.
    ///
    /// # Safety
    ///
    /// No more than N elements must be initialized.
    #[inline]
    pub(crate) const unsafe fn push_back_unchecked(&mut self, item: T)
    {
        // SAFETY: If `initialized` was correct before and the caller does not
        // invoke this method more than N times then writes will be in-bounds
        // and slots will not be initialized more than once.
        unsafe {
            self.array_mut.get_unchecked_mut(self.initialized.end).write(item);
            self.initialized.end = self.initialized.end.unchecked_add(1);
        }
    }

    /// Adds an item to the array and updates the initialized item counter.
    ///
    /// # Safety
    ///
    /// No more than N elements must be initialized.
    #[allow(unused)]
    #[inline]
    pub(crate) const unsafe fn push_front_unchecked(&mut self, item: T)
    {
        // SAFETY: If `initialized` was correct before and the caller does not
        // invoke this method more than N times then writes will be in-bounds
        // and slots will not be initialized more than once.
        unsafe {
            self.initialized.start = self.initialized.start.unchecked_sub(1);
            self.array_mut.get_unchecked_mut(self.initialized.start).write(item);
        }
    }

    #[inline]
    pub(crate) const unsafe fn pop_back_unchecked(&mut self) -> T
    {
        // SAFETY: If `initialized` was correct before and the caller does not
        // invoke this method more than N times then writes will be in-bounds
        // and slots will not be initialized more than once.
        unsafe {
            self.initialized.end = self.initialized.end.unchecked_sub(1);
            self.array_mut.get_unchecked_mut(self.initialized.end).assume_init_read()
        }
    }

    #[inline]
    pub(crate) const unsafe fn pop_front_unchecked(&mut self) -> T
    {
        // SAFETY: If `initialized` was correct before and the caller does not
        // invoke this method more than N times then writes will be in-bounds
        // and slots will not be initialized more than once.
        unsafe {
            let out = self.array_mut.get_unchecked_mut(self.initialized.start).assume_init_read();
            self.initialized.start = self.initialized.start.unchecked_add(1);
            out
        }
    }
}
impl<T> const Drop for Guard<'_, T>
where
    T: ~const Destruct
{
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.initialized.end <= self.array_mut.len());
        debug_assert!(self.initialized.start <= self.initialized.end);

        // SAFETY: this slice will contain only initialized objects.
        unsafe {
            self.array_mut.get_unchecked_mut(self.initialized.start..self.initialized.end).assume_init_drop();
        }
    }
}

/// A trait for bulks whose length can be determined at compile-time.
/// 
/// # Safety
/// 
/// We must guarantee that the bulk will always yield an exact amount of elements.
/// The bulk's length must always be the same as the length of [`Self::Array`](StaticBulk::Array).
#[rustc_on_unimplemented(
    message = "cannot determine the length of bulk `{Self}` at compile-time",
    label = "the bulk `{Self}` is not statically sized",
)]
pub unsafe trait StaticBulk: Bulk + BulkLength<Length: Array> + Sized
{
    type Array<U>: const Array<Elem = U> + Length;
}