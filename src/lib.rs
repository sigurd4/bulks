#![cfg_attr(not(test), no_std)]
#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(unboxed_closures)]
#![feature(exact_size_is_empty)]
#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]
#![feature(iter_map_windows)]
#![feature(tuple_trait)]
#![feature(try_trait_v2)]
#![feature(ptr_metadata)]
#![feature(const_trait_impl)]
#![feature(const_cmp)]
#![feature(core_intrinsics)]
#![feature(const_default)]
#![feature(const_clone)]
#![feature(const_destruct)]
#![feature(fn_traits)]
#![feature(const_convert)]
#![feature(async_fn_traits)]
#![feature(impl_trait_in_assoc_type)]
#![feature(const_index)]
#![feature(try_trait_v2_residual)]
#![feature(maybe_uninit_slice)]
#![feature(const_try)]
#![feature(slice_as_array)]
#![feature(const_drop_in_place)]
#![feature(const_precise_live_drops)]
#![feature(control_flow_into_value)]
#![feature(string_into_chars)]
#![feature(new_range_api)]
#![feature(const_option_ops)]
#![feature(step_trait)]
#![feature(iterator_try_collect)]
#![feature(doc_notable_trait)]
#![feature(mem_copy_fn)]
#![feature(const_closures)]
#![feature(specialization)]
#![feature(generic_const_exprs)]

//! Composable bulk-iteration.
//! 
//! This crate adds [`Bulk`]s, which are similar to iterators, except they are stricter. They can only be wholly consumed, where every value is operated on in bulk. This, unlike with classic [`Iterator`]s, makes them fully compatible with arrays!
//! 
//! # Constraints
//! 
//! Bulks are subject to some extra constraints that don't affect normal iterators.
//! In order for a bulk to be evaluated as is, the whole bulk must be consumed.
//! Alternatively, it can be converted into an iterator to evaluate each iteration seperately.
//! While iterators can be mutably exhausted, bulks cannot, and are therefore guaranteed to be intact.
//! 
//! Their constrained nature means fewer operations are possible, but the guarantees it gives makes it possible to use them with arrays while still retaining the array's length.
//! Operations that preserves the length of the data like [`map`](Bulk::map), [`zip`](Bulk::zip), [`enumerate`](Bulk::enumerate), [`rev`](Bulk::rev) and [`inspect`](Bulk::inspect) are allowed.
//! By enabling the [`generic_const_exprs`](https://github.com/rust-lang/rust/issues/76560)-feature, some other length-modifying operations are also allowed such as [`flat_map`](Bulk::flat_map), [`flatten`](Bulk::flatten), [`intersperse`](Bulk::intersperse), [`array_chunks`](Bulk::array_chunks) and [`map_windows`](Bulk::map_windows)
//! since these modify the bulk's length in a predetermined way.
//! Of course, wholly consuming operations like [`fold`](Bulk::fold), [`try_fold`](Bulk::try_fold), [`reduce`](Bulk::reduce), [`try_reduce`](Bulk::reduce), [`collect`](Bulk::collect) and [`try_collect`](Bulk::try_collect) are fully supported.
//! There's also [`collect_array`](Bulk::collect_array) and [`try_collect_array`](Bulk::try_collect_array) to avoid turbofish-syntax when doing [`collect`](Bulk::collect) or [`try_collect`](Bulk::try_collect).
//! 
//! Any `Bulk` that was created from an array can be collected back into an array, given that the operations done on it makes the length predetermined at compile-time.
//! Bulks can also be used with other structures, allowing generic implementations that work the same on arrays as with other iterables.
//! 
//! # Bulk
//!
//! The trait [`Bulk`] is similar to [`Iterator`], but lacks the [`next`](Iterator::next) method.
//! Instead, its function is based on the [`for_each`](Bulk::for_each) and [`try_for_each`](Bulk::try_for_each) methods.
//!
//! ```
//! trait Bulk: IntoIterator
//! {
//!     fn len(&self) -> usize;
//! 
//!     fn for_each<F>(self, f: F)
//!     where
//!         Self: Sized,
//!         F: FnMut(Self::Item);
//! 
//!     fn try_for_each<F, R>(self, f: F) -> R
//!     where
//!         Self: Sized,
//!         F: FnMut(Self::Item) -> R,
//!         R: Try<Output = ()>;
//! }
//! ```
//!
//! [`Bulk`]'s full definition includes a number of other methods as well,
//! but they are default methods, built on top of [`for_each`](Bulk::for_each) and [`try_for_each`](Bulk::try_for_each), and so you get
//! them for free.
//!
//! Bulks are also composable, and it's common to chain them together to do
//! more complex forms of processing. See the [Adapters](#adapters) section
//! below for more details.
//!
//! # The three forms of bulk-iteration
//!
//! There are three common methods which can create bulks from a collection:
//!
//! * [`bulk()`](AsBulk::bulk), which iterates over `&T`.
//! * [`bulk_mut()`](AsBulk::bulk_mut), which iterates over `&mut T`.
//! * [`into_bulk()`](IntoBulk::into_bulk), which iterates over `T`.
//! 
//! These are the in-bulk counterparts of `iter()`, `iter_mut()` and [`into_iter()`](IntoIterator::into_iter).
//! The trait [`IntoBulk`] provides the method [`into_bulk`](IntoBulk::into_bulk).
//! 
//! [`bulk()`](AsBulk::bulk) is available for any `T` where `&T` implements [`IntoBulk`], and
//! [`bulk_mut()`](AsBulk::bulk_mut) is available for any `T` where `&mut T` is [`IntoBulk`].
//! They are just shorthand for doing [`into_bulk`](IntoBulk::into_bulk) on a reference.
//! 
//! [`IntoBulk`] is automatically implemented for all [`Bulk`]s.
//! Other types that can be converted into an [`ExactSizeIterator`] through [`IntoIterator`] also automatically implement [`IntoBulk`],
//! converting them to a [`bulks::iter::Bulk`](crate::iter::Bulk), however this implementation can be specialized.
//! For example, arrays specialize this implementation, converting to [`bulks::array::IntoBulk`](crate::array::IntoBulk) instead.
//! Specializing [`IntoBulk`] is useful for collections whose length must be retained at compile-time, like arrays.
//!
//! # Implementing Bulk
//! 
//! Making your own bulk is a bit similar to making an [`Iterator`], but a little bit less convenient.
//! 
//! Your bulk needs a corresponding iterator that it can be converted to, which must be an [`ExactSizeIterator`].
//!
//! ```
//! use bulks::*;
//! 
//! /// An iterator which counts from one to `N`
//! struct CounterIter<const N: usize>
//! {
//!     count: usize,
//! }
//!
//! // we want our count to start at one, so let's add a new() method to help.
//! // This isn't strictly necessary, but is convenient.
//! impl<const N: usize> CounterIter<N>
//! {
//!     pub fn new() -> Self
//!     {
//!         CounterIter { count: 0 }
//!     }
//! }
//!
//! // Then, we implement `Iterator` for our `CounterIter`:
//! impl<const N: usize> Iterator for CounterIter<N>
//! {
//!     // We will be counting with usize
//!     type Item = usize;
//!
//!     // next() is the only required method
//!     fn next(&mut self) -> Option<Self::Item>
//!     {
//!         // Increment our count. This is why we started at zero.
//!         self.count += 1;
//!
//!         // Check to see if we've finished counting or not.
//!         if self.count <= N
//!         {
//!             Some(self.count)
//!         }
//!         else
//!         {
//!             None
//!         }
//!     }
//! 
//!     // Since we're implementing `ExactSizeIterator`, it's a good idea to override `size_hint`.
//!     fn size_hint(&self) -> (usize, Option<usize>)
//!     {
//!         let len = self.len();
//!         (len, Some(len))
//!     }
//! }
//! 
//! // We also need our `CounterIter` to be an `ExactSizeIterator`.
//! impl<const N: usize> ExactSizeIterator for CounterIter<N>
//! {
//!     fn len(&self) -> usize
//!     {
//!         N.saturating_sub(self.count)
//!     }
//! }
//! 
//! // Now that we have an iterator we can start defining our bulk-iterator.
//! 
//! /// A bulk which counts from one to five
//! struct Counter<const N: usize>;
//! 
//! // Then, we implement `IntoIterator` for our `Counter`:
//! impl<const N: usize> IntoIterator for Counter<N>
//! {
//!     // We will be counting with usize
//!     type Item = usize;
//!     // This is iterator needs to be equivalent to our bulk.
//!     type IntoIter = CounterIter<N>;
//! 
//!     fn into_iter(self) -> Self::IntoIter
//!     {
//!         CounterIter::new()
//!     }
//! }
//! 
//! // Then, we implement `Bulk` for our `Counter`:
//! impl<const N: usize> Bulk for Counter<N>
//! {
//!     fn len(&self) -> usize
//!     {
//!         N
//!     }
//! 
//!     fn for_each<F>(self, f: F)
//!     where
//!         Self: Sized,
//!         F: FnMut(Self::Item)
//!     {
//!         for i in self
//!         {
//!             f(i)
//!         }
//!     }
//! 
//!     fn try_for_each<F, R>(self, f: F) -> R
//!     where
//!         Self: Sized,
//!         F: FnMut(Self::Item) -> R,
//!         R: Try<Output = ()>
//!     {
//!         for i in self
//!         {
//!             f(i)?
//!         }
//!         R::from_output(())
//!     }
//! }
//! 
//! // To allow collection into arrays, we can implement `StaticBulk`.
//! // SAFETY: We must guarantee that `Counter<N>` will always yield exactly `N` elements.
//! unsafe impl<const N: usize> StaticBulk for Counter<N>
//! {
//!     type Array<U> = [U; N];
//! }
//!
//! // And now we can use it!
//! let counter = Counter::<5>::new();
//! let result: [_; _] = counter.collect();
//! 
//! assert_eq!(result, [1, 2, 3, 4, 5]);
//! ```
//!
//! # Adapters
//! 
//! Just like with iterators there are adapters for bulks.
//! These are functions which take a [`Bulk`] and return another [`Bulk`].
//!
//! Common bulk adapters include [`map`](Bulk::map), [`take`](Bulk::take), and [`rev`](Bulk::rev).
//! For more, see their documentation.
//!
//! # Laziness
//!
//! Bulks (and bulk [adapters](#adapters)), just like iterators, are *lazy*. This means that
//! just creating a bulk doesn't _do_ a whole lot. Nothing really happens
//! until you consume it. This is sometimes a source of confusion when
//! creating a bulk solely for its side effects. For example, the [`map`](Bulk::map)
//! method calls a closure on each element it iterates over:
//!
//! ```
//! # #![allow(unused_must_use)]
//! # #![allow(map_unit_fn)]
//! let a = [1, 2, 3, 4, 5];
//! a.bulk().map(|x| println!("{x}"));
//! ```
//!
//! This will not print any values, as we only created a bulk, rather than
//! using it. The compiler will warn us about this kind of behavior:
//!
//! ```text
//! warning: unused result that must be used: bulks are lazy and
//! do nothing unless consumed
//! ```
//!
//! The idiomatic way to write a [`map`](Bulk::map) for its side effects is to use a
//! `for` loop or call the [`for_each`](Bulk::for_each) method:
//!
//! ```
//! let a = [1, 2, 3, 4, 5];
//!
//! a.bulk().for_each(|x| println!("{x}"));
//! // or
//! for x in &v
//! {
//!     println!("{x}");
//! }
//! ```
//!
//! Another common way to evaluate a bulk is to use the [`collect`](Bulk::collect)
//! method to produce a new collection.
//! 
//! ```
//! let a = [1, 2, 3, 4, 5];
//! 
//! let b: [_; _] = a.into_bulk().collect();
//! 
//! assert_eq!(a, b);
//! ```

moddef::moddef!(
    flat(pub) mod {
        adapters,
        impl_array,
        impl_iter,
        bulk,
        double_ended_bulk,
        from_bulk,
        into_bulk,
        static_bulk,
        try_from_bulk
    },
    mod util
);

#[cfg(test)]
mod tests
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [1, 2, 3];

        let vec = a.into_bulk().zip(0..).collect::<Vec<_>>();

        println!("{vec:?}")
    }
}