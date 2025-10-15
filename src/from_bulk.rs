use crate::{util::{BulkLength, CollectLength, Length}, Bulk, IntoBulk, StaticBulk};

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
/// let five_fives = std::iter::repeat(5).take::<5>();
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
/// let five_fives = std::iter::repeat(5).take::<5>();
///
/// let v: [i32; _] = five_fives.collect();
///
/// assert_eq!(v, vec![5, 5, 5, 5, 5]);
/// ```
///
/// Using [`FromBulk::from_bulk()`] as a more readable alternative to
/// [`Bulk::collect()`]:
///
/// ```
/// use bulks::*;
/// 
/// let first = (0..10).collect::<[_; _]>();
/// let second = <[_; _]>::from_bulk(0..10);
///
/// assert_eq!(first, second);
/// ```
///
/// Implementing `FromBulk` for your type:
///
/// ```
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
/// // and we'll implement FromIterator
/// impl FromBulk<i32> for MyCollection
/// {
///     fn from_bulk<I: IntoBulk<Item = i32>>(iter: I) -> Self
///     {
///         let mut c = MyCollection::new();
///
///         for i in iter {
///             c.add(i);
///         }
///
///         c
///     }
/// }
///
/// // Now we can make a new iterator...
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
        all(A = "{integer}", any(Self = "&[{integral}]", Self = "&[{A}]", Self = "&[_]")),
        message = "a slice of type `{Self}` cannot be collected since we need to store the elements somewhere",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        any(Self = "[{A}]", Self = "[_]"),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(A = "{integer}", any(Self = "[{integral}]", Self = "[{A}]", Self = "[_]")),
        message = "a slice of type `{Self}` cannot be collected since `{Self}` has no definite size",
        label = "try explicitly collecting into a `Vec<{A}>`",
    ),
    on(
        all(any(L = "[{A}]", L = "[_]"), any(Self = "[{A}; _]", Self = "[{A}; {n}]", Self = "[_; _]", Self = "[_; {n}]")),
        message = "an array of type `{Self}` cannot be collected directly from a dynamically sized bulk",
        label = "try collecting into a `Vec<{A}>`, then using `.try_into()`",
    ),
    on(
        all(L = "[{integer}]", A = "{integer}", any(Self = "[{integral}; _]", Self = "[{integral}; {n}]", Self = "[{A}; _]", Self = "[{A}; {n}]", Self = "[_; _]", Self = "[_; {n}]")),
        message = "an array of type `{Self}` cannot be collected directly from a dynamically sized bulk",
        label = "try collecting into a `Vec<{A}>`, then using `.try_into()`",
    ),
    on(
        any(L = "[{A}]", L = "[_]"),
        message = "a value of type `{Self}` cannot be collected from a dynamically sized bulk \
                of elements of type `{A}`",
        label = "value of type `{Self}` cannot be collected from `{B}` of dynamic length"
    ),
    message = "a value of type `{Self}` cannot be collected from a bulk \
               of elements of type `{A}`",
    label = "value of type `{Self}` cannot be collected from `{B}`"
)]
pub const trait FromBulk<A, B, L = <Self as CollectLength<A>>::Length>: Sized
where
    B: BulkLength<Item = A>,
    L: Length<Elem = A> + ?Sized
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
    /// let five_fives = std::iter::repeat(5).take::<5>();
    ///
    /// let v = <[_; _]>::from_bulk(five_fives);
    ///
    /// assert_eq!(v, [5, 5, 5, 5, 5]);
    /// ```
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<Item = A, IntoBulk = B>;
}

impl<A, B, T> FromBulk<A, B, [A]> for T
where
    T: FromIterator<A>,
    B: Bulk<Item = A>
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: IntoBulk<Item = A, IntoBulk = B>
    {
        bulk.into_iter().collect()
    }
}
impl<A, B, const N: usize> const FromBulk<A, B, [A; N]> for [A; N]
where
    B: ~const StaticBulk<Item = A, Array = [A; N]>
{
    fn from_bulk<I>(bulk: I) -> Self
    where
        I: ~const IntoBulk<Item = A, IntoBulk = B>
    {
        bulk.into_bulk().collect_array()
    }
}