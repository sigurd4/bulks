use core::{marker::Destruct, mem::MaybeUninit, ops::{ControlFlow, FromResidual, Residual, Try}};

use array_trait::AsSlice;

use crate::{util::{CollectLength, Length, LengthSpec}, ArrayChunks, Chain, Cloned, Copied, DoubleEndedBulk, Enumerate, FlatMap, Flatten, FromBulk, Guard, Inspect, Intersperse, IntersperseWith, IntoBulk, IntoContained, IntoContainedBy, Map, MapWindows, Mutate, Rev, Skip, StaticBulk, StepBy, Take, Zip};

pub const trait Bulk: ~const IntoBulk<IntoBulk = Self, IntoIter: ExactSizeIterator>
{
    /// Returns the exact length of the bulk.
    ///
    /// This method has a default implementation, so you usually should not
    /// implement it directly. However, if you can provide a more efficient
    /// implementation, you can do so. See the [trait-level] docs for an
    /// example.
    ///
    /// This function has the same safety guarantees as the
    /// [`Iterator::size_hint`] function.
    /// 
    /// Similar to [`ExactSizeIterator::len`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// // a finite range knows exactly how many times it will iterate
    /// let mut range = (0..5).into_bulk();
    ///
    /// let len = range.len();
    /// 
    /// assert_eq!(len, 5);
    /// ```
    #[track_caller]
    fn len(&self) -> usize;

    /// Returns `true` if the iterator is empty.
    ///
    /// This method has a default implementation using
    /// [`Bulk::len()`], so you don't need to implement it yourself.
    /// 
    /// Similar to [`ExactSizeIterator::is_empty`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let one_element = bulks::once(0);
    /// assert!(!one_element.is_empty());
    /// ```
    #[inline]
    #[track_caller]
    fn is_empty(&self) -> bool
    {
        self.len() == 0
    }

    /// Returns the first value, and discards the rest of the bulk.
    /// 
    /// Returns [`None`] if the bulk is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let a = [1, 2, 3];
    ///
    /// let bulk = a.bulk();
    /// 
    /// let a1 = bulk.first();
    /// assert_eq!(a1, Some(1));
    /// ```
    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        const fn break_on_first<T>(x: T) -> ControlFlow<T>
        {
            ControlFlow::Break(x)
        }

        match self.try_for_each(break_on_first)
        {
            ControlFlow::Break(first) => Some(first),
            ControlFlow::Continue(()) => None
        }
    }

    /// Returns the last value, and discards the rest of the bulk.
    /// 
    /// Returns [`None`] if the bulk is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let a = [1, 2, 3];
    ///
    /// let bulk = a.bulk();
    /// 
    /// let a1 = bulk.last();
    /// assert_eq!(a1, Some(3));
    /// ```
    fn last(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        const fn store<T>(_: T, x: T) -> T
        where
            T: ~const Destruct
        {
            x
        }

        self.reduce(store)
    }

    /// Returns the `n`-th value, and discards the rest of the bulk.
    /// 
    /// Returns [`None`] if index `n` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// let a = [1, 2, 3];
    ///
    /// let bulk = a.bulk();
    /// 
    /// // The bulk is consumed, so it must be cloned each time. Don't actually do this.
    /// let a1 = bulk.clone().first();
    /// let a2 = bulk.clone().nth(2);
    /// let a3 = bulk.clone().nth(3);
    /// let a4 = bulk.clone().nth(4);
    /// 
    /// assert_eq!(a1, Some(1));
    /// assert_eq!(a2, Some(2));
    /// assert_eq!(a3, Some(3));
    /// assert_eq!(a4, None);
    /// ```
    fn nth<L>(self, n: L) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        L: ~const LengthSpec
    {
        self.skip(n).first()
    }

    /// Calls a closure on each element of a bulk.
    ///
    /// This is equivalent to using a [`for`] loop on the bulk, although
    /// `break` and `continue` are not possible from a closure. It's generally
    /// more idiomatic to use a `for` loop, but `for_each` may be more legible
    /// when processing items at the end of longer iterator chains. In some
    /// cases `for_each` may also be faster than a loop, because it will use
    /// internal iteration on adapters like [`Chain`](crate::Chain).
    ///
    /// [`for`]: ../../book/ch03-05-control-flow.html#looping-through-a-collection-with-for
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let a = [1, 2, 3, 4];
    /// let mut x0 = 0;
    /// a.into_bulk()
    ///     .for_each(|x| {
    ///         assert_eq!(x, x0 + 1);
    ///         x0 = x;
    ///     })
    /// ```
    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct;

    /// A bulk method that applies a fallible function to each item in the
    /// bulk, stopping at the first error and returning that error.
    ///
    /// This can also be thought of as the fallible form of [`for_each()`](Bulk::for_each)
    /// or as the stateless version of [`try_fold()`](Bulk::try_fold).
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let a = ["1", "2", "3", "4", "wrong"];
    /// let mut x0 = 0;
    /// let res = a.into_bulk()
    ///     .try_for_each(|x| {
    ///         let x = x.parse::<u32>().map_err(|_| x)?;
    ///         assert_eq!(x, x0 + 1);
    ///         x0 = x;
    ///         Ok(())
    ///     });
    /// assert_eq!(res, Err("wrong"))
    /// ```
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>;

    /// TODO
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        B: ~const Destruct,
        F: ~const FnMut(B, Self::Item) -> B + ~const Destruct
    {
        struct Closure<'a, B, F>
        {
            z: &'a mut Option<B>,
            f: F
        }
        impl<'a, B, F, T> const FnOnce<(T,)> for Closure<'a, B, F>
        where
            B: ~const Destruct,
            F: ~const FnOnce(B, T) -> B,
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                let Self { z, f } = self;
                let zz = unsafe {z.take().unwrap_unchecked()};
                let _ = z.insert((f)(zz, x));
            }
        }
        impl<'a, B, F, T> const FnMut<(T,)> for Closure<'a, B, F>
        where
            B: ~const Destruct,
            F: ~const FnMut(B, T) -> B,
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                let Self { z, f } = self;
                let zz = unsafe {z.take().unwrap_unchecked()};
                let _ = z.insert((f)(zz, x));
            }
        }

        let mut z = Some(init);
        self.for_each(Closure {
            z: &mut z,
            f
        });

        unsafe {
            z.unwrap_unchecked()
        }
    }

    /// TODO
    fn try_fold<B, F, R>(self, init: B, f: F) -> R
    where
        B: ~const Destruct,
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(B, Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = B, Residual: ~const Destruct>
    {
        struct Closure<'a, B, F>
        {
            z: &'a mut Option<B>,
            f: F
        }
        impl<'a, B, F, T, R> const FnOnce<(T,)> for Closure<'a, B, F>
        where
            B: ~const Destruct,
            F: ~const FnOnce(B, T) -> R,
            R: ~const Try<Output = B, Residual: ~const Destruct>
        {
            type Output = ControlFlow<R::Residual, ()>;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                let Self { z, f } = self;
                let zz = unsafe {z.take().unwrap_unchecked()};
                let _ = z.insert(f(zz, x).branch()?);
                ControlFlow::Continue(())
            }
        }
        impl<'a, B, F, T, R> const FnMut<(T,)> for Closure<'a, B, F>
        where
            B: ~const Destruct,
            F: ~const FnMut(B, T) -> R,
            R: ~const Try<Output = B, Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                let Self { z, f } = self;
                let zz = unsafe {z.take().unwrap_unchecked()};
                let _ = z.insert(f(zz, x).branch()?);
                ControlFlow::Continue(())
            }
        }

        let mut z = Some(init);
        match self.try_for_each(Closure {
            z: &mut z,
            f
        })
        {
            ControlFlow::Break(residual) => R::from_residual(residual),
            ControlFlow::Continue(()) => R::from_output(unsafe {
                z.unwrap_unchecked()
            })
        }
    }

    /// TODO
    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where 
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item, Self::Item) -> Self::Item + ~const Destruct
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T> const FnOnce<(Option<T>, T)> for Closure<F>
        where
            F: ~const FnOnce(T, T) -> T + ~const Destruct
        {
            type Output = Option<T>;

            extern "rust-call" fn call_once(self, (z, x): (Option<T>, T)) -> Self::Output
            {
                let Self { f } = self;

                match z
                {
                    Some(z) => Some(f(z, x)),
                    None => Some(x)
                }
            }
        }
        impl<F, T> const FnMut<(Option<T>, T)> for Closure<F>
        where
            F: ~const FnMut(T, T) -> T
        {
            extern "rust-call" fn call_mut(&mut self, (z, x): (Option<T>, T)) -> Self::Output
            {
                let Self { f } = self;

                match z
                {
                    Some(z) => Some(f(z, x)),
                    None => Some(x)
                }
            }
        }

        self.fold(None, Closure {
            f
        })
    }

    /// TODO
    fn try_reduce<F, R>(self, f: F) -> <R::Residual as Residual<Option<R::Output>>>::TryType
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item, Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = Self::Item, Residual: Residual<Option<Self::Item>, TryType: ~const Try> + ~const Destruct>
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T, R> const FnOnce<(Option<T>, T)> for Closure<F>
        where
            F: ~const FnOnce(T, T) -> R + ~const Destruct,
            R: ~const Try<Output = T, Residual: ~const Destruct>
        {
            type Output = ControlFlow<R::Residual, Option<T>>;

            extern "rust-call" fn call_once(self, (z, x): (Option<T>, T)) -> Self::Output
            {
                let Self { f } = self;

                ControlFlow::Continue(Some(
                    match z
                    {
                        Some(z) => f(z, x).branch()?,
                        None => x
                    }
                ))
            }
        }
        impl<F, T, R> const FnMut<(Option<T>, T)> for Closure<F>
        where
            F: ~const FnMut(T, T) -> R,
            R: ~const Try<Output = T, Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (z, x): (Option<T>, T)) -> Self::Output
            {
                let Self { f } = self;

                ControlFlow::Continue(Some(
                    match z
                    {
                        Some(z) => f(z, x).branch()?,
                        None => x
                    }
                ))
            }
        }

        match self.try_fold(None, Closure {
            f
        })
        {
            ControlFlow::Break(residual) => FromResidual::from_residual(residual),
            ControlFlow::Continue(output) => Try::from_output(output)
        }
    }
    
    /// Creates a bulk starting at the same point, but stepping by
    /// the given amount at each iteration.
    /// 
    /// Similar to [`Iterator::step_by`].
    ///
    /// # Panics
    ///
    /// The method will panic if the given step is `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let a = [0, 1, 2, 3, 4, 5];
    /// 
    /// let mut bulk = a.into_bulk().step_by([(); 2]);
    /// let a_even = bulk.collect::<[_; _]>();
    ///
    /// assert_eq!(a_even, [0, 2, 4]);
    /// 
    /// let mut bulk = a.into_bulk().skip([(); 1]).step_by([(); 2]);
    /// let a_odd = bulk.collect::<[_; _]>();
    /// 
    /// assert_eq!(a_odd, [1, 3, 5]);
    /// ```
    #[inline]
    #[track_caller]
    fn step_by<L>(self, step: L) -> StepBy<Self, L::Length<Self::Item>>
    where
        Self: Sized,
        L: ~const LengthSpec
    {
        StepBy::new(self, step)
    }

    /// Takes two bulks and creates a new bulk over both in sequence.
    ///
    /// In other words, it links two bulks together, in a chain. 🔗
    /// 
    /// Similar to [`Iterator::chain`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let s1 = b"abc".into_bulk();
    /// let s2 = b"def".into_bulk();
    ///
    /// let mut bulk = s1.chain(s2);
    /// 
    /// let s = bulk.collect();
    /// 
    /// assert_eq!(s, b"abcdef");
    /// ```
    ///
    /// Since the argument to [`chain()`](Bulk::chain) uses [`IntoBulk`], we can pass
    /// anything that can be converted into a [`Bulk`], not just a
    /// [`Bulk`] itself. For example, arrays (`[T]`) implement
    /// [`IntoBulk`], and so can be passed to [`chain()`](Bulk::chain) directly:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let a1 = [1, 2, 3];
    /// let a2 = [4, 5, 6];
    ///
    /// let mut bulk = a1.into_bulk().chain(a2);
    /// 
    /// let a = bulk.collect();
    /// 
    /// assert_eq!(a, [1, 2, 3, 4, 5, 6]);
    /// ```
    #[inline]
    #[track_caller]
    fn chain<U>(self, other: U) -> Chain<Self, U::IntoBulk>
    where
        Self: Sized,
        U: ~const IntoBulk<Item = Self::Item>,
    {
        Chain::new(self, other.into_bulk())
    }

    /// 'Zips up' two bulks or iterators into a single bulk of pairs. One of them must be a bulk.
    /// 
    /// Similar to [`Iterator::zip`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let s1 = b"abc".into_bulk().copied();
    /// let s2 = b"def".into_bulk().copied();
    ///
    /// let mut bulk = s1.zip(s2);
    /// 
    /// let s = bulk.collect::<[_; _]>();
    /// 
    /// assert_eq!(s, [(b'a', b'd'), (b'b', b'e'), (b'c', b'f')]);
    /// ```
    ///
    /// Since the argument to [`zip()`](Bulk::zip) uses [`IntoBulk`], we can pass
    /// anything that can be converted into a [`Bulk`], not just a
    /// [`Bulk`] itself. For example, arrays (`[T]`) implement
    /// [`IntoBulk`], and so can be passed to [`zip()`](Bulk::zip) directly:
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let a1 = [1, 2, 3];
    /// let a2 = [4, 5, 6];
    ///
    /// let mut bulk = a1.into_bulk().zip(a2);
    ///
    /// let a = bulk.collect::<[_; _]>();
    /// assert_eq!(a, [(1, 4), (2, 5), (3, 6)]);
    /// ```
    ///
    /// `zip()` is often used to zip an infinite iterator to a finite one.
    /// This works because the finite iterator will eventually return [`None`],
    /// ending the zipper. Zipping with `(0..)` can look a lot like [`enumerate`]:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let enumerate: [_; _] = (*b"foo").into_bulk().enumerate().collect();
    ///
    /// let zipper: Vec<_> = bulks::rzip(0.., *b"foo").collect();
    /// 
    /// assert_eq!((0, b'f'), enumerate[0]);
    /// assert_eq!((0, b'f'), zipper[0]);
    /// 
    /// assert_eq!((1, b'o'), enumerate[1]);
    /// assert_eq!((1, b'o'), zipper[1]);
    /// 
    /// assert_eq!((2, b'o'), enumerate[2]);
    /// assert_eq!((2, b'o'), zipper[2]);
    /// ```
    ///
    /// It can be more readable to use [`bulks::zip`](crate::zip):
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    ///
    /// let a = [1, 2, 3];
    /// let b = [2, 3, 4];
    ///
    /// let mut zipped = bulks::zip(
    ///     a.into_bulk().map(|x| x * 2).skip([(); 1]),
    ///     b.into_bulk().map(|x| x * 2).skip([(); 1]),
    /// );
    /// 
    /// let c = zipped.collect::<[_; _]>();
    /// assert_eq!(c, [(4, 6), (6, 8)]);
    /// ```
    ///
    /// compared to:
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// # use bulks::*;
    /// #
    /// # let a = [1, 2, 3];
    /// # let b = [2, 3, 4];
    /// #
    /// let mut zipped = a.into_bulk()
    ///     .map(|x| x * 2)
    ///     .skip([(); 1])
    ///     .zip(b.into_bulk()
    ///         .map(|x| x * 2)
    ///         .skip([(); 1])
    ///     );
    /// #
    /// # let c = zipped.collect::<[_; _]>();
    /// # assert_eq!(c, [(4, 6), (6, 8)]);
    /// ```
    #[inline]
    #[track_caller]
    fn zip<U>(self, other: U) -> Zip<Self, <<U as IntoContained>::IntoContained as IntoBulk>::IntoBulk>
    where
        Self: Sized,
        U: ~const IntoContainedBy<Self>
    {
        crate::zip(self, other)
    }

    /// Creates a new bulk which places a copy of `separator` between adjacent
    /// items of the original bulk.
    /// 
    /// Similar to [`Iterator::intersperse`].
    ///
    /// In case `separator` does not implement [`Clone`](core::clone::Clone) or needs to be
    /// computed every time, use [`intersperse_with`](Bulk::intersperse_with).
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let mut a = [0, 1, 2].into_bulk().intersperse(100).collect::<[_; _]>();
    /// 
    /// assert_eq!(a, [0, 100, 1, 100, 2]);
    /// ```
    ///
    /// `intersperse` can be very useful to join a bulk's items using a common element:
    /// ```
    /// use bulks::*;
    ///
    /// let words = ["Hello", "World", "!"];
    /// let hello: String = words.into_bulk().intersperse(" ").collect();
    /// assert_eq!(hello, "Hello World !");
    /// ```
    #[inline]
    #[track_caller]
    fn intersperse(self, separator: Self::Item) -> Intersperse<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        Intersperse::new(self, separator)
    }

    /// Creates a new bulk which places an item generated by `separator`
    /// between adjacent items of the original bulk.
    ///
    /// The closure will be called exactly once each time an item is placed
    /// between two adjacent items from the underlying bulk; specifically,
    /// the closure is not called if the underlying bulk has less than
    /// two items.
    /// 
    /// Similar to [`Iterator::intersperse_with`].
    ///
    /// If the bulk's item implements [`Clone`](core::clone::Clone), it may be easier to use
    /// [`intersperse`](Bulk::intersperse).
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    ///
    /// #[derive(PartialEq, Debug)]
    /// struct NotClone(usize);
    ///
    /// let v = [NotClone(0), NotClone(1), NotClone(2)];
    /// let u = v.into_bulk().intersperse_with(|| NotClone(99)).collect::<[_; _]>();
    ///
    /// assert_eq!(u, [NotClone(0), NotClone(99), NotClone(1), NotClone(99), NotClone(2)]);
    /// ```
    ///
    /// [`intersperse_with`](Bulk::intersperse_with) can be used in situations where the separator needs
    /// to be computed:
    /// ```
    /// use bulks::*;
    ///
    /// let src = ["Hello", "to", "all", "people", "!!"].bulk().copied();
    ///
    /// // The closure mutably borrows its context to generate an item.
    /// let mut happy_emojis = [" ❤️ ", " 😀 "].into_iter();
    /// let separator = || happy_emojis.next().unwrap_or(" 🦀 ");
    ///
    /// let result = src.intersperse_with(separator).collect::<String>();
    /// assert_eq!(result, "Hello ❤️ to 😀 all 🦀 people 🦀 !!");
    /// ```
    #[inline]
    #[track_caller]
    fn intersperse_with<G>(self, separator: G) -> IntersperseWith<Self, G>
    where
        Self: Sized,
        G: FnMut() -> Self::Item,
    {
        IntersperseWith::new(self, separator)
    }

    /// Takes a closure and creates a bulk which calls that closure on each
    /// element.
    ///
    /// [`map()`](Bulk::map) transforms one bulk into another, by means of its argument:
    /// something that implements [`FnMut`]. It produces a new bulk which
    /// calls this closure on each element of the original bulk.
    /// 
    /// Similar to [`Iterator::map`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let a = [1, 2, 3];
    ///
    /// let mut b = a.bulk().map(|x| 2 * x).collect::<[_; _]>();
    ///
    /// assert_eq!(b, [2, 4, 6]);
    /// ```
    ///
    /// If you're doing some sort of side effect, prefer [`for`] to [`map()`](Bulk::map):
    ///
    /// ```
    /// # #![allow(unused_must_use)]
    /// use bulks::*;
    /// 
    /// // don't do this:
    /// (0..5).into_bulk().map(|x| println!("{x}"));
    ///
    /// // it won't even execute, as it is lazy. Rust will warn you about this.
    ///
    /// // Instead, use a for-loop:
    /// for x in (0..5).into_bulk()
    /// {
    ///     println!("{x}");
    /// }
    /// ```
    /// 
    /// [`for`]: ../../book/ch03-05-control-flow.html#looping-through-a-collection-with-for
    #[inline]
    #[track_caller]
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        Map::new(self, f)
    }

    /// Creates a bulk which gives the current index together with its values.
    ///
    /// The bulk returned yields pairs `(i, val)`, where `i` is the
    /// current index of iteration and `val` is its corresponding value.
    /// 
    /// Similar to [`Iterator::enumerate`].
    ///
    /// [`enumerate()`](Bulk::enumerate) keeps its count as a [`usize`]. If you want to count by a
    /// different sized integer, the [`zip`](Bulk::zip) function provides similar
    /// functionality.
    ///
    /// # Overflow Behavior
    ///
    /// The method does no guarding against overflows, so enumerating more than
    /// [`usize::MAX`] elements either produces the wrong result or panics. If
    /// overflow checks are enabled, a panic is guaranteed.
    ///
    /// # Panics
    ///
    /// The returned bulk might panic if the to-be-returned index would
    /// overflow a [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// let a = ['a', 'b', 'c'];
    ///
    /// let b = a.into_bulk().enumerate().collect();
    ///
    /// assert_eq!(b, [(0, 'a'), (1, 'b'), (2, 'c')]);
    /// ```
    #[inline]
    #[track_caller]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized
    {
        Enumerate::new(self)
    }

    /// Creates a bulk that skips the first `n` elements.
    /// 
    /// Similar to [`Iterator::skip`].
    ///
    /// [`skip(n)`](Bulk::skip) skips elements until `n` elements are skipped or the end of the
    /// bulk is reached (whichever happens first). The returned bulk will yield the remaining elements.
    /// If the original bulk is too short, then the returned bulk is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let a = [1, 2, 3];
    ///
    /// let b = a.into_bulk().skip([(); 2]).collect::<[_; _]>();
    /// let c = a.into_bulk().skip(2).collect::<Vec<_>>();
    ///
    /// assert_eq!(b, [3]);
    /// assert_eq!(c, [3]);
    /// ```
    #[inline]
    #[track_caller]
    fn skip<L>(self, n: L) -> Skip<Self, L::Length<Self::Item>>
    where
        Self: Sized,
        L: ~const LengthSpec
    {
        Skip::new(self, n)
    }

    /// Creates a bulk for the first `n` elements, or fewer
    /// if the underlying bulk/iterator is shorter.
    ///
    /// [`take(n)`](Bulk::take) yields elements until `n` elements are yielded or the end of the
    /// bulk is reached (whichever happens first).
    /// The returned bulk is a prefix of length `n` if the original bulk/iterator
    /// contains at least `n` elements, otherwise it contains all of the
    /// (fewer than `n`) elements of the original bulk/iterator.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let a = [1, 2, 3];
    ///
    /// let b = a.into_bulk().take([(); 3]).collect::<Vec<_>>();
    ///
    /// assert_eq!(b, [1, 2]);
    /// ```
    ///
    /// `take()` is often used with an infinite iterator, to make it finite:
    ///
    /// ```
    /// let a = (0..).take(3).collect::<Vec<_>>();
    ///
    /// assert_eq!(a, [0, 1, 2])
    /// ```
    ///
    /// If less than `n` elements are available,
    /// [`take`](Bulk::take) will limit itself to the size of the underlying bulk/iterator:
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let v = [1, 2];
    /// let b = v.into_bulk().take([(); 5]).collect::<[_; _]>();
    /// 
    /// assert_eq!(b, [1, 2])
    /// ```
    #[doc(alias = "limit")]
    #[inline]
    #[track_caller]
    fn take<L>(self, n: L) -> Take<Self, L::Length<Self::Item>>
    where
        Self: Sized,
        L: LengthSpec
    {
        Take::new(self, n)
    }

    /// Creates a bulk that works like map, but flattens nested structure.
    ///
    /// The [`map`](Bulk::map) adapter is very useful, but only when the closure
    /// argument produces values. If it produces something iterable instead, there's
    /// an extra layer of indirection. [`flat_map()`](Bulk::flat_map) will remove this extra layer
    /// on its own.
    /// 
    /// Similar to [`Iterator::flat_map`].
    ///
    /// You can think of `flat_map(f)` as the semantic equivalent
    /// of [`map`](Bulk::map)ping, and then [`flatten`](Bulk::flatten)ing as in `map(f).flatten()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let words = ["alpha", "beta", "gamma"];
    ///
    /// let merged: String = words.bulk()
    ///                           .flat_map(|s| s.chars())
    ///                           .collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    #[inline]
    #[track_caller]
    fn flat_map<U, F>(self, f: F) -> FlatMap<Self, U, F>
    where
        Self: Sized,
        U: IntoBulk<IntoBulk: StaticBulk>,
        F: FnMut(Self::Item) -> U,
    {
        FlatMap::new(self, f)
    }

    /// Creates a bulk that flattens nested structure.
    ///
    /// This is useful when you have a bulk of bulk or a bulk of
    /// things that can be turned into bulks and you want to remove one
    /// level of indirection.
    /// 
    /// Similar to [`Iterator::flatten`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let data = [[1, 2, 3], [4, 5, 6]];
    /// let flattened: [_; _] = data.into_bulk().flatten().collect();
    /// assert_eq!(flattened, [1, 2, 3, 4, 5, 6]);
    /// ```
    ///
    /// Mapping and then flattening:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let words = ["alpha", "beta", "gamma"];
    ///
    /// let merged: String = words.bulk()
    ///                           .map(|s| s.chars())
    ///                           .flatten()
    ///                           .collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    ///
    /// You can also rewrite this in terms of [`flat_map()`](Bulk::flat_map), which is preferable
    /// in this case since it conveys intent more clearly:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let words = ["alpha", "beta", "gamma"];
    ///
    /// let merged: String = words.iter()
    ///                           .flat_map(|s| s.chars())
    ///                           .collect();
    /// assert_eq!(merged, "alphabetagamma");
    /// ```
    ///
    /// Flattening only removes one level of nesting at a time:
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    /// 
    /// let d3 = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
    ///
    /// let d2: [_; _] = d3.into_bulk().flatten().collect();
    /// assert_eq!(d2, [[1, 2], [3, 4], [5, 6], [7, 8]]);
    ///
    /// let d1: [_; _] = d3.into_bulk().flatten().flatten().collect();
    /// assert_eq!(d1, [1, 2, 3, 4, 5, 6, 7, 8]);
    /// ```
    ///
    /// Here we see that [`flatten()`](Bulk::flatten) does not perform a "deep" flatten.
    /// Instead, only one level of nesting is removed. That is, if you
    /// [`flatten()`](Bulk::flatten) a three-dimensional array, the result will be
    /// two-dimensional and not one-dimensional. To get a one-dimensional
    /// structure, you have to [`flatten()`](Bulk::flatten) again.
    #[inline]
    #[track_caller]
    fn flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Item: IntoBulk<IntoBulk: StaticBulk>,
    {
        Flatten::new(self)
    }

    /// Calls the given function `f` for each contiguous window of size `N` over
    /// `self` and returns a bulk of the outputs of `f`. The windows during mapping will overlap.
    /// 
    /// Similar to [`Iterator::map_windows`].
    ///
    /// In the following example, the closure is called three times with the
    /// arguments `&['a', 'b']`, `&['b', 'c']` and `&['c', 'd']` respectively.
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    ///
    /// let strings = b"abcd".bulk()
    ///     .map(|&c| char::from(c))
    ///     .map_windows(|[x, y]| format!("{}+{}", x, y))
    ///     .collect::<[_; _]>();
    ///
    /// assert_eq!(strings, ["a+b", "b+c", "c+d"]);
    /// ```
    ///
    /// Note that the const parameter `N` is usually inferred by the
    /// destructured argument in the closure.
    ///
    /// The returned bulk yields 𝑘 − `N` + 1 items (where 𝑘 is the number of
    /// items yielded by `self`). If 𝑘 is less than `N`, this method yields an
    /// empty bulk.
    ///
    /// # Panics
    ///
    /// Panics if `N` is zero.
    ///
    /// ```should_panic
    /// use bulks::*;
    ///
    /// let bulk = [0].into_bulk().map_windows(|&[]| ());
    /// ```
    ///
    /// # Examples
    ///
    /// Building the sums of neighboring numbers.
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    ///
    /// let w = [1, 3, 8, 1].bulk()
    ///     .map_windows(|&[a, b]| a + b)
    ///     .collect::<[_; _]>();
    /// 
    /// assert_eq!(w, [1 + 3, 3 + 8, 8 + 1]);
    /// ```
    ///
    /// Since the elements in the following example implement [`Copy`], we can
    /// just copy the array and get a bulk of the windows.
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    ///
    /// let w = b"ferris".bulk()
    ///     .map_windows(|w: &[_; 3]| w.bulk().copied().copied().collect::<[_; _]>())
    ///     .collect::<[_; _]>();
    /// 
    /// assert_eq!(w, [[b'f', b'e', b'r'], [b'e', b'r', b'r'], [b'r', b'r', b'i'], [b'r', b'i', b's']]);
    /// ```
    ///
    /// You can also use this function to check the sortedness of a bulk.
    /// For the simple case, rather use [`Bulk::is_sorted`].
    ///
    /// ```
    /// # #![feature(generic_const_exprs)]
    /// use bulks::*;
    ///
    /// let w = [0.5, 1.0, 3.5, 3.0, 8.5, 8.5, f32::NAN].bulk()
    ///     .map_windows(|[a, b]| a <= b)
    ///     .collect::<[_; _]>();
    /// 
    /// assert_eq!(w, [true, true, false, true, true, false]);
    /// ```
    #[inline]
    #[track_caller]
    fn map_windows<F, R, const N: usize>(self, f: F) -> MapWindows<Self, F, N>
    where
        Self: Sized,
        F: FnMut(&[Self::Item; N]) -> R,
    {
        MapWindows::new(self, f)
    }

    /// Does something with each element of a bulk, passing the value on.
    ///
    /// When using bulks, you'll often chain several of them together.
    /// While working on such code, you might want to check out what's
    /// happening at various parts in the pipeline. To do that, insert
    /// a call to [`inspect()`](Bulk::inspect).
    /// 
    /// Similar to [`Iterator::inspect`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let a = [1, 4, 2, 3];
    ///
    /// // this iterator sequence is complex.
    /// let sum = a.bulk()
    ///     .cloned()
    ///     .filter(|x| x % 2 == 0)
    ///     .fold(0, |sum, i| sum + i);
    ///
    /// println!("{sum}");
    ///
    /// // let's add some inspect() calls to investigate what's happening
    /// let sum = a.bulk()
    ///     .cloned()
    ///     .inspect(|x| println!("about to filter: {x}"))
    ///     .filter(|x| x % 2 == 0)
    ///     .inspect(|x| println!("made it through filter: {x}"))
    ///     .fold(0, |sum, i| sum + i);
    ///
    /// println!("{sum}");
    /// ```
    ///
    /// This will print:
    ///
    /// ```text
    /// 6
    /// about to filter: 1
    /// about to filter: 4
    /// made it through filter: 4
    /// about to filter: 2
    /// made it through filter: 2
    /// about to filter: 3
    /// 6
    /// ```
    ///
    /// Logging errors before discarding them:
    ///
    /// ```
    /// let lines = ["1", "2", "a"];
    ///
    /// let sum: i32 = lines
    ///     .iter()
    ///     .map(|line| line.parse::<i32>())
    ///     .inspect(|num| {
    ///         if let Err(ref e) = *num {
    ///             println!("Parsing error: {e}");
    ///         }
    ///     })
    ///     .filter_map(Result::ok)
    ///     .sum();
    ///
    /// println!("Sum: {sum}");
    /// ```
    ///
    /// This will print:
    ///
    /// ```text
    /// Parsing error: invalid digit found in string
    /// Sum: 3
    /// ```
    #[inline]
    #[track_caller]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item),
    {
        Inspect::new(self, f)
    }

    /// Mutates with each element of a bulk, passing the value on.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let a = [1, 4, 2, 3];
    ///
    /// // this iterator sequence is complex.
    /// let b = a.into_bulk()
    ///     .mutate(|x| *x += 1)
    ///     .collect::<[_; _]>();
    ///
    /// assert_eq!(b, [2, 5, 3, 4]);
    /// ```
    #[inline]
    #[track_caller]
    fn mutate<F>(self, f: F) -> Mutate<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Self::Item),
    {
        Mutate::new(self, f)
    }

    /// Transforms a bulk into a collection.
    ///
    /// [`collect()`](Bulk::collect) can take anything bulkable, and turn it into a relevant
    /// collection.
    /// 
    /// Similar to [`Iterator::collect`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let a = [1, 2, 3];
    ///
    /// let doubled: [i32; 3] = a.bulk()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(doubled, [2, 4, 6]);
    /// ```
    ///
    /// Note that we needed the `: [i32; 3]` on the left-hand side. This is because
    /// we could collect into, for example, a [`VecDeque<T>`](std::collections::VecDeque) instead:
    ///
    /// ```
    /// use std::collections::VecDeque;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let doubled: VecDeque<i32> = a.bulk()
    ///     .map(|x| x * 2)
    ///     .collect();
    ///
    /// assert_eq!(doubled[0], 2);
    /// assert_eq!(doubled[1], 4);
    /// assert_eq!(doubled[2], 6);
    /// ```
    ///
    /// Using the 'turbofish' instead of annotating `doubled`:
    ///
    /// ```
    /// let a = [1, 2, 3];
    ///
    /// let doubled = a.bulk().map(|x| x * 2).collect::<[i32; 3]>();
    ///
    /// assert_eq!(doubled, [2, 4, 6]);
    /// ```
    ///
    /// Because `collect()` only cares about what you're collecting into, you can
    /// still use a partial type hint, `_`, with the turbofish:
    ///
    /// ```
    /// let a = [1, 2, 3];
    ///
    /// let doubled = a.bulk().map(|x| x * 2).collect::<[_; _]>();
    ///
    /// assert_eq!(doubled, [2, 4, 6]);
    /// ```
    ///
    /// Using `collect()` to make a [`String`](std::string::String):
    ///
    /// ```
    /// let chars = ['g', 'd', 'k', 'k', 'n'];
    ///
    /// let hello: String = chars.bulk()
    ///     .map(|x| x as u8)
    ///     .map(|x| (x + 1) as char)
    ///     .collect();
    ///
    /// assert_eq!(hello, "hello");
    /// ```
    ///
    /// If you have a list of [`Result<T, E>`][`Result`]s, you can use `collect()` to
    /// see if any of them failed:
    ///
    /// ```
    /// let results = [Ok(1), Err("nope"), Ok(3), Err("bad")];
    ///
    /// let result: Result<[_; _], &str> = results.into_bulk().collect();
    ///
    /// // gives us the first error
    /// assert_eq!(result, Err("nope"));
    ///
    /// let results = [Ok(1), Ok(3)];
    ///
    /// let result: Result<[_; _], &str> = results.into_bulk().collect();
    ///
    /// // gives us the list of answers
    /// assert_eq!(result, Ok([1, 3]));
    /// ```
    #[inline]
    #[track_caller]
    #[must_use = "if you really need to exhaust the bulk, consider `.for_each(drop)` instead"]
    #[allow(invalid_type_param_default)]
    fn collect<B, L = <B as CollectLength<<Self as IntoIterator>::Item>>::Length>(self) -> B
    where
        Self: Sized,
        B: ~const FromBulk<Self::Item, Self, L>,
        L: Length<Elem = Self::Item> + ?Sized
    {
        FromBulk::from_bulk(self)
    }

    fn collect_array(self) -> <Self as StaticBulk>::Array<<Self as IntoIterator>::Item>
    where
        Self: StaticBulk
    {
        let mut array = MaybeUninit::<Self::Array<Self::Item>>::uninit();
        let array_mut = unsafe {
            array.as_mut_ptr().cast::<Self::Array<MaybeUninit<Self::Item>>>().as_mut().unwrap().as_mut_slice()
        };
        let mut guard = Guard { array_mut, initialized: 0..0 };

        struct Closure<'a, 'b, T>
        {
            guard: &'a mut Guard<'b, T>
        }

        impl<'a, 'b, T> const FnOnce<(T,)> for Closure<'a, 'b, T>
        {
            type Output = ();

            extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
            {
                self.call_mut(args)
            }
        }
        impl<'a, 'b, T> const FnMut<(T,)> for Closure<'a, 'b, T>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                unsafe {
                    self.guard.push_back_unchecked(x);
                }
            }
        }

        let pusher = Closure {
            guard: &mut guard
        };

        self.for_each(pusher);
        
        core::mem::forget(guard);
        unsafe {
            MaybeUninit::assume_init(array)
        }
    }

    fn try_collect_array(self) -> <<Self::Item as Try>::Residual as Residual<Self::Array<<Self::Item as Try>::Output>>>::TryType
    where
        Self: StaticBulk<Item: ~const Destruct + ~const Try<Residual: Residual<(), TryType: ~const Try> + Residual<Self::Array<<Self::Item as Try>::Output>, TryType: ~const Try> + ~const Destruct, Output: ~const Destruct>> + ~const Bulk
    {
        let mut array = MaybeUninit::<Self::Array<<Self::Item as Try>::Output>>::uninit();
        let array_mut = unsafe {
            array.as_mut_ptr().cast::<Self::Array<MaybeUninit<<Self::Item as Try>::Output>>>().as_mut().unwrap().as_mut_slice()
        };
        let mut guard = Guard { array_mut, initialized: 0..0 };

        struct Closure<'a, 'b, T>
        where
            T: Try<Residual: Residual<()>>
        {
            guard: &'a mut Guard<'b, <T as Try>::Output>
        }

        impl<'a, 'b, T> const FnOnce<(T,)> for Closure<'a, 'b, T>
        where
            T: ~const Try<Residual: Residual<(), TryType: ~const Try>>
        {
            type Output = <<T as Try>::Residual as Residual<()>>::TryType;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                unsafe {
                    self.guard.push_back_unchecked(x?);
                }
                Try::from_output(())
            }
        }
        impl<'a, 'b, T> const FnMut<(T,)> for Closure<'a, 'b, T>
        where
            T: ~const Try<Residual: Residual<(), TryType: ~const Try>>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                unsafe {
                    self.guard.push_back_unchecked(x?);
                }
                Try::from_output(())
            }
        }

        let pusher = Closure {
            guard: &mut guard
        };

        self.try_for_each(pusher)?;
        
        core::mem::forget(guard);
        unsafe {
            Try::from_output(MaybeUninit::assume_init(array))
        }
    }

    /// Reverses a bulks's direction.
    ///
    /// Usually, bulks span from left to right. After using `rev()`,
    /// a bulk will instead span from right to left.
    /// 
    /// Similar to [`Iterator::rev`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let b: [_; _] = a.into_bulk().rev().collect();
    ///
    /// assert_eq!(b, [3, 2, 1]);
    /// ```
    #[inline]
    #[track_caller]
    #[doc(alias = "reverse")]
    fn rev(self) -> Rev<Self>
    where
        Self: Sized,
        Self: DoubleEndedBulk
    {
        Rev::new(self)
    }

    /// Creates a bulk which copies all of its elements.
    ///
    /// This is useful when you have a bulk of `&T`, but you need a
    /// bulk of `T`.
    /// 
    /// Similar to [`Iterator::copied`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let v_copied: [_; _] = a.bulk().copied().collect();
    ///
    /// // copied is the same as .map(|&x| x)
    /// let v_map: [_; _] = a.bulk().map(|&x| x).collect();
    ///
    /// assert_eq!(v_copied, [1, 2, 3]);
    /// assert_eq!(v_map, [1, 2, 3]);
    /// ```
    #[inline]
    #[track_caller]
    fn copied<'a, T>(self) -> Copied<Self>
    where
        T: Copy + 'a,
        Self: Sized + ~const Bulk<Item = &'a T>,
    {
        Copied::new(self)
    }

    /// Creates a bulk which [`clone`](Clone::clone)s all of its elements.
    ///
    /// This is useful when you have a bulk of `&T`, but you need a
    /// bulk of `T`.
    ///
    /// There is no guarantee whatsoever about the `clone` method actually
    /// being called *or* optimized away. So code should not depend on
    /// either.
    /// 
    /// Similar to [`Iterator::cloned`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let v_cloned: [_; _] = a.bulk().cloned().collect();
    ///
    /// // cloned is the same as .map(|&x| x), for integers
    /// let v_map: [_; _] = a.bulk().map(|&x| x).collect();
    ///
    /// assert_eq!(v_cloned, [1, 2, 3]);
    /// assert_eq!(v_map, [1, 2, 3]);
    /// ```
    #[inline]
    #[track_caller]
    fn cloned<'a, T>(self) -> Cloned<Self>
    where
        T: Clone + 'a,
        Self: Sized + ~const Bulk<Item = &'a T>,
    {
        Cloned::new(self)
    }

    /// Returns a bulk of `N` elements of the bulk at a time.
    ///
    /// The chunks do not overlap. If `N` does not divide the length of the
    /// bulk, then the last up to `N-1` elements will be omitted or the remainder
    /// can then be retrieved from [`.into_remainder()`][crate::ArrayChunks::into_remainder]
    /// or [`.collect_with_remainder()`][crate::ArrayChunks::collect_with_remainder]
    /// 
    /// Similar to [`Iterator::array_chunks`].
    ///
    /// # Panics
    ///
    /// Panics if `N` is zero.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let bulk = "lorem".bulk().array_chunks();
    /// let (c, r) = bulk.collect_with_remainder();
    /// 
    /// assert_eq!(c, [['l', 'o'], ['r', 'e']]);
    /// assert_eq!(r, ['m']);
    /// ```
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let data = [1, 1, 2, -2, 6, 0, 3, 1];
    /// //          ^-----^  ^------^
    /// for [x, y, z] in data.bulk().array_chunks()
    /// {
    ///     assert_eq!(x + y + z, 4);
    /// }
    /// ```
    #[inline]
    #[track_caller]
    fn array_chunks<const N: usize>(self) -> ArrayChunks<Self, N>
    where
        Self: Sized,
    {
        ArrayChunks::new(self)
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn test_reduce()
    {
        let a = [1, 5, -3, 7, 9, 3, -1, 3];

        let sum = a.into_bulk().reduce(|a, b| a + b).unwrap_or(0);
        let product = a.into_bulk().reduce(|a, b| a*b).unwrap_or(1);
        let min = a.into_bulk().reduce(|a, b| a.min(b)).unwrap();
        let max = a.into_bulk().reduce(|a, b| a.max(b)).unwrap();
        let mean = sum as f32/a.len() as f32;
        let variance = a.into_bulk().map(|a| a as f32 - mean).map(|a| a*a).reduce(|a, b| a + b).unwrap_or(0.0).sqrt();

        println!("sum = {sum}");
        println!("product = {product}");
        println!("min = {min}");
        println!("max = {max}");

        println!("mean = {mean}");
        println!("variance = {variance}");
    }
}