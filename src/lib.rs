#![cfg_attr(not(test), no_std)]
#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(extend_one)]
#![feature(unboxed_closures)]
#![feature(const_range)]
#![feature(exact_size_is_empty)]
#![feature(iter_array_chunks)]
#![feature(allocator_api)]
#![feature(inplace_iteration)]
#![feature(const_try_residual)]
#![feature(iter_intersperse)]
#![feature(ascii_char)]
#![feature(iter_map_windows)]
#![feature(tuple_trait)]
#![feature(try_trait_v2)]
#![feature(ptr_metadata)]
#![feature(range_bounds_is_empty)]
#![feature(const_trait_impl)]
#![feature(trusted_len)]
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
#![feature(const_try)]
#![feature(const_drop_in_place)]
#![feature(const_precise_live_drops)]
#![feature(control_flow_into_value)]
#![feature(trait_alias)]
#![feature(new_range_api)]
#![feature(const_option_ops)]
#![feature(step_trait)]
#![feature(iterator_try_collect)]
#![feature(doc_notable_trait)]
#![feature(mem_copy_fn)]
#![feature(array_into_iter_constructors)]
#![feature(decl_macro)]
#![feature(negative_impls)]
#![feature(ptr_as_ref_unchecked)]
#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]
#![feature(maybe_uninit_uninit_array_transpose)]
#![feature(const_result_trait_fn)]
#![feature(associated_type_defaults)]
#![feature(macro_metavar_expr_concat)]
#![feature(const_eval_select)]
#![feature(const_closures)]
#![feature(specialization)]
#![feature(generic_const_exprs)]
#![allow(clippy::map_clone)] // Temporary, because Option::cloned is not const and clippy doesn't get that

//! Composable bulk-iteration.
//!
//! This crate adds [`Bulk`]s, which are similar to iterators, except they are stricter. They can only be wholly consumed, where every value is operated on in bulk. This,
//! unlike with classic [`Iterator`]s, makes them fully compatible with arrays!
//!
//! # Example
//!
//! ```
//! use bulks::*;
//!
//! let a = [1, 2, 3];
//!
//! let b: [_; _] = a.bulk()
//!     .copied()
//!     .map(|x| (x - 1) as usize)
//!     .enumerate()
//!     .inspect(|(i, x)| assert_eq!(i, x))
//!     .collect();
//!
//! assert_eq!(b, [(0, 0), (1, 1), (2, 2)]);
//! ```
//!
//! # Constraints
//!
//! Bulks are subject to some extra constraints that don't affect normal iterators.
//! In order for a bulk to be evaluated as is, the whole bulk must be consumed.
//! Alternatively, it can be converted into an iterator to evaluate each iteration seperately.
//! While iterators can be mutably exhausted, bulks cannot, and are therefore guaranteed to be intact.
//!
//! Their constrained nature means fewer operations are possible, but the guarantees it gives makes it possible to use them with arrays while still retaining the array's
//! length. Operations that preserves the length of the data like [`map`](Bulk::map), [`zip`](Bulk::zip), [`enumerate`](Bulk::enumerate), [`rev`](Bulk::rev) and
//! [`inspect`](Bulk::inspect) are allowed. By enabling the [`generic_const_exprs`](https://github.com/rust-lang/rust/issues/76560)-feature, some other length-modifying operations are also allowed such as [`flat_map`](Bulk::flat_map), [`flatten`](Bulk::flatten), [`intersperse`](Bulk::intersperse), [`array_chunks`](Bulk::array_chunks) and [`map_windows`](Bulk::map_windows)
//! since these modify the bulk's length in a predetermined way.
//! Of course, wholly consuming operations like [`fold`](Bulk::fold), [`try_fold`](Bulk::try_fold), [`reduce`](Bulk::reduce), [`try_reduce`](Bulk::reduce),
//! [`collect`](Bulk::collect) and [`try_collect`](Bulk::try_collect) are fully supported. There's also [`collect_array`](Bulk::collect_array) and
//! [`try_collect_array`](Bulk::try_collect_array) to avoid turbofish-syntax when doing [`collect`](Bulk::collect) or [`try_collect`](Bulk::try_collect).
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
//! # #![feature(try_trait_v2)]
//! use core::ops::Try;
//!
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
//! # #![feature(try_trait_v2)]
//! use core::ops::Try;
//!
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
//!     type MinLength = [(); N];
//!     type MaxLength = [(); N];
//!
//!     fn len(&self) -> usize
//!     {
//!         N
//!     }
//!
//!     fn for_each<F>(self, mut f: F)
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
//!     fn try_for_each<F, R>(self, mut f: F) -> R
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
//! // And now we can use it!
//! let counter = Counter::<5>;
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
//! use bulks::*;
//!
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
//! use bulks::*;
//!
//! let a = [1, 2, 3, 4, 5];
//!
//! a.bulk().for_each(|x| println!("{x}"));
//! // or
//! for x in &a
//! {
//!     println!("{x}");
//! }
//! ```
//!
//! Another common way to evaluate a bulk is to use the [`collect`](Bulk::collect)
//! method to produce a new collection.
//!
//! ```
//! use bulks::*;
//!
//! let a = [1, 2, 3, 4, 5];
//!
//! let b: [_; _] = a.into_bulk().collect();
//!
//! assert_eq!(a, b);
//! ```

/*
# MISSING FEATURES:
- collect_into (requires `Extend` to become a const-trait)
- const enumerate_with
*/

#[cfg(feature = "alloc")]
extern crate alloc;

moddef::moddef!(
    flat(pub) mod {
        adapters,
        impl_array,
        impl_iter,
        impl_range,
        impl_slice,
        impl_vec for cfg(feature = "alloc"),
        impl_option,
        bulk,
        collect_nearest,
        double_ended_bulk,
        from_bulk,
        into_bulk,
        split_bulk,
        random_access_bulk,
        static_bulk
    },
    mod util
);

pub use util::ConstStep as Step;

#[cfg(false)]
pub mod asm
{
    use crate::{Bulk, IntoBulk};

    #[unsafe(no_mangle)]
    pub fn asm_bulk(a: [u8; 4]) -> [u8; 4]
    {
        a.into_bulk().collect()
    }

    #[unsafe(no_mangle)]
    pub fn asm_swap(a: [u8; 4]) -> [u8; 4]
    {
        let mut bulk = a.into_bulk();

        bulk.swap_inplace(0, 1);

        bulk.collect()
    }

    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn asm_bench_swap(mut a: [u8; 4]) -> [u8; 4]
    {
        a.swap(0, 1);

        a
    }
}

#[cfg(test)]
mod tests
{
    use crate::{option::MaybeBulk, *};

    #[test]
    fn it_works()
    {
        let a: [i32; _] = [1, 2, 3];

        let f = |x| (x - 1) as usize;

        let b = a.bulk().copied().map(f).enumerate().inspect(|(i, x)| assert_eq!(i, x)).collect_nearest();

        assert_eq!(b, [(0, 0), (1, 1), (2, 2)] as [(usize, usize); _]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn nearest()
    {
        let a: [i32; _] = [1, 2, 3];

        let f = |x| (x - 1) as usize;

        let b = a.bulk().copied().map(f).enumerate().inspect(|(i, x)| assert_eq!(i, x));

        fn nearest<T>(bulk: T) -> <T as CollectNearest>::Nearest
        where
            T: Bulk
        {
            bulk.collect_nearest()
        }

        let b = nearest(b);

        assert_eq!(b, [(0, 0), (1, 1), (2, 2)] as [(usize, usize); _]);
    }

    #[test]
    fn test_option()
    {
        let a: [i32; _] = [1];
        let b: [i32; _] = [];

        let a: Option<i32> = a.into_bulk().map(|x| x + 1).collect();
        let b: Option<i32> = b.into_bulk().map(|x| x + 1).collect();

        assert_eq!(a, Some(2));
        assert_eq!(b, None);

        fn maybe(_: impl IntoBulk<IntoBulk: MaybeBulk<Item = i32>>) {}

        maybe(Some(1));
        maybe([1; 0]);
        maybe([1]);
    }

    #[test]
    fn test_swap() -> Result<(), Box<dyn std::error::Error>>
    {
        let a = [1, 2, 3, 4];

        let mut bulk = a.into_bulk();

        bulk.try_swap_inplace(0, 3)?;
        bulk.try_swap_inplace(1, 2)?;

        let b = bulk.rev().collect_array();

        assert_eq!(a, b);
        println!("{a:?}");

        Ok(())
    }
}
