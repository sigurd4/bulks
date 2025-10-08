use crate::Bulk;

pub trait LimitToBulk: IntoIterator
{
    /// 'Zips up' two bulks or iterators into a single bulk of pairs. One of them must be a bulk.
    /// 
    /// Analogous to [`Iterator::zip`].
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
    /// let mut bulk = s1.zip(s2);
    /// 
    /// let s = bulk.collect();
    /// 
    /// assert_eq!(s, [(b'a', b'd'), (b'b', b'e'), (b'c', b'f')]);
    /// ```
    ///
    /// Since the argument to [`zip()`](LimitToBulk::zip) uses [`IntoBulk`], we can pass
    /// anything that can be converted into a [`Bulk`], not just a
    /// [`Bulk`] itself. For example, arrays (`[T]`) implement
    /// [`IntoBulk`], and so can be passed to [`zip()`](LimitToBulk::zip) directly:
    ///
    /// ```
    /// use bulks::*;
    /// 
    /// let a1 = [1, 2, 3];
    /// let a2 = [4, 5, 6];
    ///
    /// let mut bulk = a1.into_bulk().zip(a2);
    ///
    /// let a = bulk.collect();
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
    /// let enumerate: [_; _] = "foo".bulk().enumerate().collect();
    ///
    /// let zipper: [_; _] = (0..).zip("foo".bulk()).collect();
    ///
    /// assert_eq!((0, 'f'), enumerate[0]);
    /// assert_eq!((0, 'f'), zipper[0]);
    ///
    /// assert_eq!((1, 'o'), enumerate[1]);
    /// assert_eq!((1, 'o'), zipper[1]);
    ///
    /// assert_eq!((2, 'o'), enumerate[2]);
    /// assert_eq!((2, 'o'), zipper[2]);
    /// ```
    ///
    /// It can be more readable to use [`bulks::zip`](crate::zip):
    ///
    /// ```
    /// use bulks::*;
    ///
    /// let a = [1, 2, 3];
    /// let b = [2, 3, 4];
    ///
    /// let mut zipped = bulks::zip(
    ///     a.into_bulk().map(|x| x * 2).skip::<1>(),
    ///     b.into_bulk().map(|x| x * 2).skip::<1>(),
    /// );
    /// 
    /// let c = zipped.collect();
    /// assert_eq!(c, [(1, 2), (2, 3), (3, 4)]);
    /// ```
    ///
    /// compared to:
    ///
    /// ```
    /// # use bulks::*;
    /// #
    /// # let a = [1, 2, 3];
    /// # let b = [2, 3, 4];
    /// #
    /// let mut zipped = a
    ///     .into_bulk()
    ///     .map(|x| x * 2)
    ///     .skip::<1>()
    ///     .zip(b.into_bulk().map(|x| x * 2).skip::<1>());
    /// #
    /// # let c = zipped.collect();
    /// # assert_eq!(c, [(2, 3), (3, 4)]);
    /// ```
    #[inline]
    #[cfg(disabled)]
    fn zip<U>(self, other: U) -> <Self as ZipToBulk<U>>::Zip
    where
        Self: Sized,
        Self: ZipToBulk<U>,
    {
        self.zip_to_bulk(other)
    }

    /// Creates a bulk for the first `n` elements, or fewer
    /// if the underlying bulk/iterator is shorter.
    ///
    /// [`take(n)`](LimitToBulk::take) yields elements until `n` elements are yielded or the end of the
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
    /// use bulk::*;
    /// 
    /// let a = [1, 2, 3];
    ///
    /// let b = a.into_bulk().take::<2>().collect();
    ///
    /// assert_eq!(b, [1, 2]);
    /// ```
    ///
    /// `take()` is often used with an infinite iterator, to make it finite:
    ///
    /// ```
    /// let a = (0..).take::<3>().collect();
    ///
    /// assert_eq!(a, [0, 1, 2])
    /// ```
    ///
    /// If less than `n` elements are available,
    /// [`take`](LimitToBulk::take) will limit itself to the size of the underlying bulk/iterator:
    ///
    /// ```
    /// use bulk::*;
    /// 
    /// let v = [1, 2];
    /// let b = v.into_bulk().take::<5>().collect();
    /// 
    /// assert_eq!(b, [1, 2])
    /// ```
    #[doc(alias = "limit")]
    #[inline]
    #[cfg(disabled)]
    fn take<const N: usize>(self) -> Take<Self, N>
    where
        Self: Sized,
    {
        Take::new(self)
    }
}

impl<T> LimitToBulk for T
where
    T: Bulk
{

}