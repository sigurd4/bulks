use core::{marker::Destruct, ops::{Residual, Try}};

use crate::{Bulk, IntoBulk, StaticBulk, option::MaybeLength};

/// Conversion from a [`Bulk`].
///
/// By implementing [`FromBulk`] for a type, you define how it will be
/// created from a bulk. This is common for types which describe a
/// collection of some kind.
///
/// If you want to create a collection from the contents of a bulk, the
/// [`Bulk::collect()`] method is preferred. However, when you need to
/// specify the container type, [`FromBulk::from_bulk()`] can be more
/// readable than using a turbofish (e.g. `::<Vec<_>>()`). See the
/// [`Bulk::collect()`] documentation for more examples of its use.
///
/// See also: [`IntoBulk`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use bulks::*;
/// 
/// let five_fives = bulks::repeat_n(5, [(); 5]);
///
/// let v = <[_; _]>::from_bulk(five_fives);
///
/// assert_eq!(v, [5, 5, 5, 5, 5]);
/// ```
///
/// Using [`Bulk::collect()`] to implicitly use `FromBulk`:
///
/// ```
/// use bulks::*;
/// 
/// let five_fives = bulks::repeat_n(5, [(); 5]);
///
/// let v: [i32; _] = five_fives.collect();
///
/// assert_eq!(v, [5, 5, 5, 5, 5]);
/// ```
///
/// Using [`FromBulk::from_bulk()`] as a more readable alternative to
/// [`Bulk::collect()`]:
///
/// ```
/// use bulks::*;
/// 
/// let first = (0..10).into_bulk().collect::<Vec<_>, _>();
/// let second = <Vec<_>>::from_bulk(0..10);
///
/// assert_eq!(first, second);
/// ```
///
/// Implementing `FromBulk` for your type:
///
/// ```
/// # #![feature(try_trait_v2)]
/// # #![feature(try_trait_v2_residual)]
/// use core::ops::{Try, Residual};
/// 
/// use bulks::*;
/// 
/// // A sample collection, that's just a wrapper over Vec<T>
/// #[derive(Debug)]
/// struct MyCollection(Vec<i32>);
///
/// // Let's give it some methods so we can create one and add things
/// // to it.
/// impl MyCollection
/// {
///     fn new() -> MyCollection
///     {
///         MyCollection(Vec::new())
///     }
///
///     fn add(&mut self, elem: i32)
///     {
///         self.0.push(elem);
///     }
/// }
///
/// // and we'll implement FromBulk
/// impl FromBulk<[i32]> for MyCollection
/// {
///     fn from_bulk<I>(bulk: I) -> Self
///     where
///         I: IntoBulk<Item = i32>
///     {
///         let mut c = MyCollection::new();
///
///         for i in bulk {
///             c.add(i);
///         }
///
///         c
///     }
/// 
///     fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
///     where
///         I: IntoBulk<Item: Try<Output = i32, Residual: Residual<Self>>>
///     {
///         let mut c = MyCollection::new();
///
///         for i in bulk {
///             c.add(i?);
///         }
///
///         Try::from_output(c)
///     }
/// }
///
/// // Now we can make a new bulk...
/// let iter = (0..5).into_bulk();
///
/// // ... and make a MyCollection out of it
/// let c = MyCollection::from_bulk(iter);
///
/// assert_eq!(c.0, vec![0, 1, 2, 3, 4]);
///
/// // collect works too!
///
/// let iter = (0..5).into_bulk();
/// let c: MyCollection = iter.collect();
///
/// assert_eq!(c.0, vec![0, 1, 2, 3, 4]);
/// ```
#[rustc_on_unimplemented(
    on(
        any(Self = "&[{A}]", Self = "&[_]"),
        message = "a slice of type `{Self}` cannot be collected since we need to store the elements somewhere",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(any(A = "[{integer}]", A = "[{integer}; _]", A = "[{integer}; {n}]", A = "Option<{integer}>"), any(Self = "&[{integral}]", Self = "&[{A}]", Self = "&[_]")),
        message = "a slice of type `{Self}` cannot be collected since we need to store the elements somewhere",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        any(Self = "[{A}]", Self = "[_]"),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(any(A = "[{integer}]", A = "[{integer}; _]", A = "[{integer}; {n}]", A = "Option<{integer}>"), any(Self = "[{integral}]", Self = "[{A}]", Self = "[_]")),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(any(A = "[{A}]", A = "[_]"), any(Self = "[{A}; _]", Self = "[{A}; {n}]", Self = "[_; _]", Self = "[_; {n}]")),
        message = "an array of type `{Self}` cannot be collected directly from a dynamically sized bulk",
        label = "try collecting into a `Vec<{A}>`, then using `.try_into()`",
    ),
    on(
        all(A = "[{integer}]", any(Self = "[{integral}; _]", Self = "[{integral}; {n}]", Self = "[{A}; _]", Self = "[{A}; {n}]", Self = "[_; _]", Self = "[_; {n}]")),
        message = "an array of type `{Self}` cannot be collected directly from a dynamically sized bulk",
        label = "try collecting into a `Vec<{A}>`, then using `.try_into()`",
    ),
    on(
        any(A = "[{A}]", A = "[_]"),
        message = "a value of type `{Self}` cannot be collected from a dynamically sized bulk \
                of elements of type `{A}`",
        label = "value of type `{Self}` cannot be collected from dynamically sized bulk"
    ),
    message = "a value of type `{Self}` cannot be collected from a bulk \
               of structure `{A}`",
    label = "value of type `{Self}` cannot be collected from bulk"
)]
pub const trait FromBulk<A>: Sized
where
    A: CollectionAdapter + ?Sized
{
    /// Creates a value from a bulk.
    ///
    /// See the [crate-level documentation](crate) for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let five_fives = bulks::repeat_n(5, [(); 5]);
    ///
    /// let v = <[_; _]>::from_bulk(five_fives);
    ///
    /// assert_eq!(v, [5, 5, 5, 5, 5]);
    /// ```
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item = <A as CollectionAdapter>::Elem>,
        A: ~const CollectionStrategy<I::IntoBulk, Self>;

    /// Fallably creates a value from a bulk.
    ///
    /// See the [crate-level documentation](crate) for more.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let five_fives = bulks::repeat_n(Some(5), [(); 5]);
    ///
    /// let v = <[_; _]>::try_from_bulk(five_fives);
    ///
    /// assert_eq!(v, Some([5, 5, 5, 5, 5]));
    /// ```
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = <A as CollectionAdapter>::Elem, Residual: ~const Residual<Self, TryType: ~const Try>> + ~const Destruct>,
        A: ~const TryCollectionAdapter<I::IntoBulk, Self>;
}

pub trait CollectionAdapter
{
    type Elem;
    type Strategy<B, T>
    where
        B: Bulk;
    type TryStrategy<B, T>
    where
        B: Bulk<Item: Try<Residual: Residual<T>>>;
}
impl<A> CollectionAdapter for [A]
{
    type Elem = A;
    type Strategy<B, T> = B
    where
        B: Bulk;
    type TryStrategy<B, T> = B
    where
        B: Bulk<Item: Try<Residual: Residual<T>>>;
}
impl<A> CollectionAdapter for Option<A>
{
    type Elem = A;
    type Strategy<B, T> = B
    where
        B: Bulk;
    type TryStrategy<B, T> = B
    where
        B: Bulk<Item: Try<Residual: Residual<T>>>;
}
impl<A, const N: usize> CollectionAdapter for [A; N]
{
    type Elem = A;
    type Strategy<B, T> = T
    where
        B: Bulk;
    type TryStrategy<B, T> = <<B::Item as Try>::Residual as Residual<T>>::TryType
    where
        B: Bulk<Item: Try<Residual: Residual<T>>>;
}

pub const trait CollectionStrategy<B, T>
where
    B: ~const Bulk + ?Sized
{
    fn adapt(bulk: B) -> Self::Strategy<B, T>
    where
        Self: CollectionAdapter<Elem = B::Item>,
        B: Sized;
}
impl<A, B, T> CollectionStrategy<B, T> for [A]
where
    B: Bulk + ?Sized
{
    fn adapt(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, T>
    where
        B: Sized
    {
        bulk
    }
}
impl<A, B, const N: usize> const CollectionStrategy<B, [A; N]> for [A; N]
where
    B: ~const Bulk<Item = A> + StaticBulk<Array<A> = [A; N]>
{
    fn adapt(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, [A; N]>
    where
        B: Sized
    {
        bulk.collect_array()
    }
}
impl<A, B, const N: usize> const CollectionStrategy<B, Option<[A; N]>> for [Option<A>; N]
where
    B: ~const Bulk<Item = Option<A>> + StaticBulk<Array<A> = [A; N]>,
    A: ~const Destruct
{
    fn adapt(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, Option<[A; N]>>
    where
        B: Sized
    {
        bulk.try_collect_array()
    }
}
impl<A, B, E, const N: usize> const CollectionStrategy<B, Result<[A; N], E>> for [Result<A, E>; N]
where
    B: ~const Bulk<Item = Result<A, E>> + StaticBulk<Array<A> = [A; N]>,
    A: ~const Destruct,
    E: ~const Destruct
{
    fn adapt(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, Result<[A; N], E>>
    where
        B: Sized
    {
        bulk.try_collect_array()
    }
}
impl<A, B> const CollectionStrategy<B, Option<A>> for Option<A>
where
    B: ~const Bulk<Item = A, MinLength: MaybeLength, MaxLength: MaybeLength, Length: MaybeLength> + ?Sized
{
    fn adapt(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, Option<A>>
    where
        B: Sized
    {
        bulk
    }
}

pub const trait TryCollectionAdapter<B, T>
where
    B: ~const Bulk<Item: ~const Try<Residual: ~const Residual<T>>>
{
    fn try_adapt(bulk: B) -> Self::TryStrategy<B, T>
    where
        Self: CollectionAdapter<Elem = <B::Item as Try>::Output>;
}
impl<A, B, T> TryCollectionAdapter<B, T> for [A]
where
    B: Bulk<Item: Try<Residual: Residual<T>>>
{
    fn try_adapt(bulk: B) -> <Self as CollectionAdapter>::TryStrategy<B, T>
    {
        bulk
    }
}
impl<A, B, R, T, Y, const N: usize> const TryCollectionAdapter<B, [T; N]> for [A; N]
where
    B: ~const Bulk<Item = R> + StaticBulk<Array<T> = [T; N]>,
    R: ~const Try<Output = T, Residual: Residual<(), TryType: ~const Try> + ~const Residual<[T; N], TryType = Y> + ~const Destruct> + ~const Destruct,
    Y: ~const Try<Residual: ~const Destruct>,
    T: ~const Destruct
{
    fn try_adapt(bulk: B) -> <Self as CollectionAdapter>::TryStrategy<B, [T; N]>
    {
        bulk.try_collect_array()
    }
}

// Collect iterator

impl<A, T> FromBulk<[A]> for T
where
    T: FromIterator<A>
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: IntoBulk<Item = A>
    {
        bulk.into_iter().collect()
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: IntoBulk<Item: Try<Output = A, Residual: Residual<Self>>>
    {
        bulk.into_iter().try_collect()
    }
}

// Collect options

impl<A> const FromBulk<Option<A>> for Option<A>
where
    A: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item = A>,
        Option<A>: ~const CollectionStrategy<I::IntoBulk, Self>
    {
        bulk.into_bulk().first()
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = A, Residual: ~const Residual<Self, TryType: ~const Try>> + ~const Destruct>,
        Option<A>: ~const TryCollectionAdapter<I::IntoBulk, Self>
    {
        Try::from_output(match bulk.into_bulk().first()
        {
            Some(result) => Some(result?),
            None => None
        })
    }
}

// Collect arrays

impl<A, const N: usize> const FromBulk<[A; N]> for [A; N]
where
    A: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk<Item = A>>,
        [A; N]: ~const CollectionStrategy<I::IntoBulk, Self>
    {
        <[A; N] as CollectionStrategy<I::IntoBulk, Self>>::adapt(bulk.into_bulk())
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = A, Residual: ~const Residual<Self, TryType: ~const Try>>>,
        [A; N]: ~const TryCollectionAdapter<I::IntoBulk, Self>
    {
        <[A; N] as TryCollectionAdapter<I::IntoBulk, Self>>::try_adapt(bulk.into_bulk())
    }
}
impl<A, const N: usize> const FromBulk<[Option<A>; N]> for Option<[A; N]>
where
    A: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk<Item = Option<A>>>,
        [Option<A>; N]: ~const CollectionStrategy<I::IntoBulk, Self>
    {
        <[Option<A>; N] as CollectionStrategy<I::IntoBulk, Self>>::adapt(bulk.into_bulk())
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = Option<A>, Residual: ~const Residual<Self, TryType: ~const Try>>>,
        [Option<A>; N]: ~const TryCollectionAdapter<I::IntoBulk, Self>
    {
        <[Option<A>; N] as TryCollectionAdapter<I::IntoBulk, Self>>::try_adapt(bulk.into_bulk())
    }
}
impl<A, E, const N: usize> const FromBulk<[Result<A, E>; N]> for Result<[A; N], E>
where
    A: ~const Destruct,
    E: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk<Item = Result<A, E>>>,
        [Result<A, E>; N]: ~const CollectionStrategy<I::IntoBulk, Self>
    {
        <[Result<A, E>; N] as CollectionStrategy<I::IntoBulk, Self>>::adapt(bulk.into_bulk())
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = Result<A, E>, Residual: ~const Residual<Self>>>,
        [Result<A, E>; N]: ~const TryCollectionAdapter<I::IntoBulk, Self>
    {
        <[Result<A, E>; N] as TryCollectionAdapter<I::IntoBulk, Self>>::try_adapt(bulk.into_bulk())
    }
}