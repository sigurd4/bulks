use crate::IntoBulk;

/// Converts the arguments to bulks and zips them.
///
/// See the documentation of [`Bulk::zip`](crate::Bulk::zip) for more.
///
/// # Examples
///
/// ```
/// use bulks::zip;
///
/// let xs = [1, 2, 3];
/// let ys = [4, 5, 6];
///
/// let bulk = zip(xs, ys);
///
/// let s = bulk.collect();
/// assert_eq!(s, [(1, 4), (2, 5), (3, 6)]);
///
/// // Nested zips are also possible:
/// let zs = [7, 8, 9];
///
/// let bulk = zip(zip(xs, ys), zs);
///
/// let s = bulk.collect();
/// assert_eq!(s, [((1, 4), 7), ((2, 5), 8), ((3, 6), 9)]);
/// ```
pub fn zip<A, B>(a: A, b: B) -> Zip<A::IntoBulk, B::IntoBulk>
where
    A: IntoBulk,
    B: IntoBulk,
{
    a.into_bulk().zip(b)
}