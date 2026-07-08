use core::{marker::Destruct, ops::{Residual, Try}};

use array_trait::length::Length;

use crate::{Bulk, IntoBulk};

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
/// struct MyCollection(Vec<usize>);
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
///     fn add(&mut self, elem: usize)
///     {
///         self.0.push(elem);
///     }
/// }
///
/// // and we'll implement FromBulk
/// impl FromBulk<[usize]> for MyCollection
/// {
///     fn from_bulk<I>(bulk: I) -> Self
///     where
///         I: IntoBulk<Item = usize>
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
///         I: IntoBulk<Item: Try<Output = usize, Residual: Residual<Self>>>
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
        all(any(A = "[{integer}]", A = "[{integer}; _]", A = "[{integer}; {integral}]", A = "Option<{integer}>"), any(Self = "&[{integral}]", Self = "&[{A}]", Self = "&[_]")),
        message = "a slice of type `{Self}` cannot be collected since we need to store the elements somewhere",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        any(Self = "[{A}]", Self = "[_]"),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(any(A = "[{integer}]", A = "[{integer}; _]", A = "[{integer}; {integral}]", A = "Option<{integer}>"), any(Self = "[{integral}]", Self = "[{A}]", Self = "[_]")),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(any(A = "[{A}]", A = "[_]"), any(Self = "[{A}; _]", Self = "[{A}; {integral}]", Self = "[_; _]", Self = "[_; {integral}]")),
        message = "an array of type `{Self}` cannot be collected directly from a dynamically sized bulk",
        label = "try collecting into a `Vec<{A}>`, then using `.try_into()`",
    ),
    on(
        all(A = "[{integer}]", any(Self = "[{integral}; _]", Self = "[{integral}; {integral}]", Self = "[{A}; _]", Self = "[{A}; {integral}]", Self = "[_; _]", Self = "[_; {integral}]")),
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
        A: ~const CollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>;

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
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = <A as CollectionAdapter>::Elem, Residual: ~const Residual<()> + ~const Residual<Self, TryType: ~const Try> + ~const Destruct> + ~const Destruct>,
        A: ~const TryCollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>;
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

pub const trait CollectionStrategy<LMIN, LMAX, A>
where
    LMIN: Length<Elem = ()> + ?Sized,
    LMAX: Length<Elem = ()> + ?Sized
{
    fn adapt<B>(bulk: B) -> Self::Strategy<B, A>
    where
        B: ~const Bulk<Item = Self::Elem, MinLength = LMIN, MaxLength = LMAX>,
        Self: CollectionAdapter;
}
impl<A, LMIN, LMAX, T> CollectionStrategy<LMIN, LMAX, T> for [A]
where
    LMIN: Length<Elem = ()> + ?Sized,
    LMAX: Length<Elem = ()> + ?Sized
{
    fn adapt<B>(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, T>
    where
        B: Bulk<Item = <Self as CollectionAdapter>::Elem, MinLength = LMIN, MaxLength = LMAX>
    {
        bulk
    }
}
const impl<A, LMIN, LMAX, const N: usize> CollectionStrategy<LMIN, LMAX, [A; N]> for [A; N]
where
    LMIN: Length<Elem = (), Intersect<LMAX> = [(); N]> + ?Sized,
    LMAX: Length<Elem = ()> + ?Sized
{
    fn adapt<B>(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, [A; N]>
    where
        B: ~const Bulk<Item = <Self as CollectionAdapter>::Elem, MinLength = LMIN, MaxLength = LMAX>
    {
        bulk.collect_array()
    }
}
const impl<A, const N: usize> CollectionStrategy<[(); N], [(); N], Option<[A; N]>> for [Option<A>; N]
where
    A: ~const Destruct
{
    fn adapt<B>(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, Option<[A; N]>>
    where
        B: ~const Bulk<Item = <Self as CollectionAdapter>::Elem, MinLength = [(); N], MaxLength = [(); N]>
    {
        bulk.try_collect_array()
    }
}
const impl<A, E, const N: usize> CollectionStrategy<[(); N], [(); N], Result<[A; N], E>> for [Result<A, E>; N]
where
    A: ~const Destruct,
    E: ~const Destruct
{
    fn adapt<B>(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, Result<[A; N], E>>
    where
        B: ~const Bulk<Item = <Self as CollectionAdapter>::Elem, MinLength = [(); N], MaxLength = [(); N]>
    {
        bulk.try_collect_array()
    }
}
const impl<LMIN, LMAX, A> CollectionStrategy<LMIN, LMAX, Option<A>> for Option<A>
where
    LMIN: Length<Elem = (), Min<[(); 1]> = LMIN> + ?Sized,
    LMAX: Length<Elem = (), Min<[(); 1]> = LMAX> + ?Sized,
{
    fn adapt<B>(bulk: B) -> <Self as CollectionAdapter>::Strategy<B, Option<A>>
    where
        B: ~const Bulk<Item = <Self as CollectionAdapter>::Elem, MinLength = LMIN, MaxLength = LMAX>
    {
        bulk
    }
}

pub const trait TryCollectionStrategy<LMIN, LMAX, A>
where 
    LMIN: Length<Elem = ()> + ?Sized,
    LMAX: Length<Elem = ()> + ?Sized
{
    fn try_adapt<B>(bulk: B) -> Self::TryStrategy<B, A>
    where
        B: ~const Bulk<Item: ~const Try<Output = <Self as CollectionAdapter>::Elem, Residual: ~const Residual<A> + ~const Residual<()> + ~const Destruct> + ~const Destruct, MinLength = LMIN, MaxLength = LMAX>,
        Self: CollectionAdapter;
}
impl<A, LMIN, LMAX, T> TryCollectionStrategy<LMIN, LMAX, T> for [A]
where
    LMIN: Length<Elem = ()> + ?Sized,
    LMAX: Length<Elem = ()> + ?Sized
{
    fn try_adapt<B>(bulk: B) -> <Self as CollectionAdapter>::TryStrategy<B, T>
    where
        B: Bulk<Item: Try<Output = <Self as CollectionAdapter>::Elem, Residual: Residual<T> + Residual<()>>, MinLength = LMIN, MaxLength = LMAX>
    {
        bulk
    }
}
const impl<A, LMIN, LMAX, const N: usize> TryCollectionStrategy<LMIN, LMAX, [A; N]> for [A; N]
where
    LMIN: Length<Elem = (), Intersect<LMAX> = [(); N]> + ?Sized,
    LMAX: Length<Elem = ()> + ?Sized,
    A: ~const Destruct
{
    fn try_adapt<B>(bulk: B) -> <Self as CollectionAdapter>::TryStrategy<B, [A; N]>
    where
        B: ~const Bulk<Item: ~const Try<Output = <Self as CollectionAdapter>::Elem, Residual: ~const Residual<[A; N]> + ~const Residual<()> + ~const Destruct> + ~const Destruct, MinLength = LMIN, MaxLength = LMAX>
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

const impl<A> FromBulk<Option<A>> for Option<A>
where
    A: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item = A>,
        Option<A>: ~const CollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>
    {
        bulk.into_bulk().first()
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = A, Residual: ~const Residual<Self, TryType: ~const Try>> + ~const Destruct>,
        Option<A>: ~const TryCollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>
    {
        Try::from_output(match bulk.into_bulk().first()
        {
            Some(result) => Some(result?),
            None => None
        })
    }
}

// Collect arrays

const impl<A, const N: usize> FromBulk<[A; N]> for [A; N]
where
    A: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk<Item = A>>,
        [A; N]: ~const CollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, [A; N]>
    {
        <[A; N]>::adapt(bulk.into_bulk())
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = A, Residual: ~const Residual<()> + ~const Residual<Self, TryType: ~const Try> + ~const Destruct> + ~const Destruct>,
        [A; N]: ~const TryCollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, [A; N]>
    {
        <[A; N]>::try_adapt(bulk.into_bulk())
    }
}
const impl<A, const N: usize> FromBulk<[Option<A>; N]> for Option<[A; N]>
where
    A: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk<Item = Option<A>>>,
        [Option<A>; N]: ~const CollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>
    {
        <[Option<A>; N]>::adapt(bulk.into_bulk())
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = Option<A>, Residual: ~const Residual<()> + ~const Residual<Self, TryType: ~const Try> + ~const Destruct> + ~const Destruct>,
        [Option<A>; N]: ~const TryCollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>
    {
        <[Option<A>; N]>::try_adapt(bulk.into_bulk())
    }
}
const impl<A, E, const N: usize> FromBulk<[Result<A, E>; N]> for Result<[A; N], E>
where
    A: ~const Destruct,
    E: ~const Destruct
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk<Item = Result<A, E>>>,
        [Result<A, E>; N]: ~const CollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>
    {
        <[Result<A, E>; N]>::adapt(bulk.into_bulk())
    }
    fn try_from_bulk<I>(bulk: I) -> <<I::Item as Try>::Residual as Residual<Self>>::TryType
    where
        I: ~const IntoBulk<IntoBulk: ~const Bulk, Item: ~const Try<Output = Result<A, E>, Residual: ~const Residual<()> + ~const Residual<Self> + ~const Destruct> + ~const Destruct>,
        [Result<A, E>; N]: ~const TryCollectionStrategy<<I::IntoBulk as Bulk>::MinLength, <I::IntoBulk as Bulk>::MaxLength, Self>
    {
        <[Result<A, E>; N]>::try_adapt(bulk.into_bulk())
    }
}