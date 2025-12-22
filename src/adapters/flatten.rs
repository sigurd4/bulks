use core::{marker::Destruct, ops::Try};

use array_trait::{AsArray, length::{self, LengthValue}};

use crate::{Bulk, DoubleEndedBulk, InplaceBulk, InplaceBulkSpec, IntoBulk, IntoContained, RandomAccessBulk, RandomAccessBulkSpec, StaticBulk};

/// A bulk that flattens one level of nesting in a of things
/// that can be turned into bulks.
///
/// This `struct` is created by the [`flatten`](Bulk::flatten) method on [`Bulk`]. See its
/// documentation for more.
#[must_use = "bulks are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    bulk: I
}

impl<I> Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    pub(crate) const fn new(bulk: I) -> Self
    {
        Self {
            bulk
        }
    }

    const fn chunk() -> usize
    {
        <<<I::Item as IntoBulk>::IntoBulk as StaticBulk>::Array<<I::Item as IntoIterator>::Item> as AsArray>::LENGTH
    }
}

impl<I> IntoIterator for Flatten<I>
where
    I: Bulk<Item: IntoBulk<IntoBulk: StaticBulk>>
{
    type Item = <I::Item as IntoIterator>::Item;
    type IntoIter = <<core::iter::Flatten<I::IntoIter> as IntoContained>::IntoContained as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        let Self { bulk } = self;
        unsafe {
            bulk.into_iter()
                .flatten()
                .into_contained()
                .into_iter()
        }
    }
}
impl<I> const Bulk for Flatten<I>
where
    I: ~const Bulk<Item: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk> + ~const Destruct>
{
    type MinLength = length::Mul<I::MinLength, <<I::Item as IntoBulk>::IntoBulk as StaticBulk>::Array<()>>;
    type MaxLength = length::Mul<I::MaxLength, <<I::Item as IntoBulk>::IntoBulk as StaticBulk>::Array<()>>;

    fn len(&self) -> usize
    {
        let Self { bulk } = self;
        bulk.len()*Self::chunk()
    }
    fn is_empty(&self) -> bool
    {
        let Self { bulk } = self;
        Self::chunk() == 0 || bulk.is_empty()
    }

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: ~const Destruct,
        Self: Sized
    {
        const fn into_first<T>(x: T) -> Option<T::Item>
        where
            T: ~const IntoBulk<Item: ~const Destruct>
        {
            x.into_bulk().first()
        }

        let Self { bulk } = self;
        bulk.first().and_then(into_first)
    }

    fn for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().for_each(self.f)
            }
        }
        impl<F, T> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.for_each(Closure {
            f
        })
    }
    fn try_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T, R> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_for_each(self.f)
            }
        }
        impl<F, T, R> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const Bulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.try_for_each(Closure {
            f
        })
    }
}
impl<I> const DoubleEndedBulk for Flatten<I>
where
    I: ~const DoubleEndedBulk<Item: ~const IntoBulk<IntoBulk: ~const DoubleEndedBulk + StaticBulk> + ~const Destruct>,
    Self::IntoIter: DoubleEndedIterator
{
    fn rev_for_each<F>(self, f: F)
    where
        Self: Sized,
        F: ~const FnMut(Self::Item) + ~const Destruct
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            type Output = ();

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().rev_for_each(self.f)
            }
        }
        impl<F, T> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk>,
            F: ~const FnMut(T::Item) + ~const Destruct
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().rev_for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.rev_for_each(Closure {
            f
        })
    }
    fn try_rev_for_each<F, R>(self, f: F) -> R
    where
        Self: Sized,
        Self::Item: ~const Destruct,
        F: ~const FnMut(Self::Item) -> R + ~const Destruct,
        R: ~const Try<Output = (), Residual: ~const Destruct>
    {
        struct Closure<F>
        {
            f: F
        }
        impl<F, T, R> const FnOnce<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            type Output = R;

            extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_rev_for_each(self.f)
            }
        }
        impl<F, T, R> const FnMut<(T,)> for Closure<F>
        where
            T: ~const IntoBulk<IntoBulk: StaticBulk + ~const DoubleEndedBulk, Item: ~const Destruct>,
            F: ~const FnMut(T::Item) -> R + ~const Destruct,
            R: ~const Try<Output = (), Residual: ~const Destruct>
        {
            extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
            {
                x.into_bulk().try_rev_for_each(&mut self.f)
            }
        }

        let Self { bulk } = self;
        bulk.try_rev_for_each(Closure {
            f
        })
    }
}
impl<I, P> const RandomAccessBulk for Flatten<I>
where
    I: ~const RandomAccessBulk<Item: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk> + ~const Destruct>,
    for<'a> &'a I::ItemPointee: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk, Item = &'a P>
{
    type ItemPointee = P;
    type EachRef<'a> = Flatten<I::EachRef<'a>>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_ref<'a>(Self { bulk }: &'a Self) -> Self::EachRef<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        bulk.each_ref()
            .flatten()
    }
}
impl<I, P> const InplaceBulk for Flatten<I>
where
    I: ~const InplaceBulk<Item: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk> + ~const Destruct>,
    for<'a> &'a I::ItemPointee: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk, Item = &'a P>,
    for<'a> &'a mut I::ItemPointee: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk, Item = &'a mut P>
{
    type EachMut<'a> = Flatten<I::EachMut<'a>>
    where
        Self::ItemPointee: 'a,
        Self: 'a;

    fn each_mut<'a>(Self { bulk }: &'a mut Self) -> Self::EachMut<'a>
    where
        Self::ItemPointee: 'a,
        Self: 'a
    {
        bulk.each_mut()
            .flatten()
    }
}

impl<I, P, const M: usize> const RandomAccessBulkSpec for Flatten<I>
where
    I: ~const RandomAccessBulk<Item: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk> + ~const Destruct>,
    for<'a> &'a I::ItemPointee: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk, Item = &'a P>,
    I::ItemPointee: ~const RandomAccessBulk<ItemPointee = P> + StaticBulk<Length = [(); M]>
{
    fn _get<'a, L>(Self { bulk }: &'a Self, i: L) -> Option<&'a <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: 'a
    {
        match bulk.get(length::value::div(i, [(); M]))
        {
            Some(item) => item.get(length::value::rem(i, [(); M])),
            None => None
        }
    }
}
impl<I, P, const M: usize> const InplaceBulkSpec for Flatten<I>
where
    I: ~const InplaceBulk<Item: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk> + ~const Destruct>,
    for<'a> &'a I::ItemPointee: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk, Item = &'a P>,
    for<'a> &'a mut I::ItemPointee: ~const IntoBulk<IntoBulk: ~const Bulk + StaticBulk, Item = &'a mut P>,
    I::ItemPointee: ~const InplaceBulk<ItemPointee = P> + StaticBulk<Length = [(); M]>
{
    fn _get_mut<'a, L>(Self { bulk }: &'a mut Self, i: L) -> Option<&'a mut <Self as RandomAccessBulk>::ItemPointee>
    where
        L: LengthValue,
        Self: 'a
    {
        match bulk.get_mut(length::value::div(i, [(); M]))
        {
            Some(item) => item.get_mut(length::value::rem(i, [(); M])),
            None => None
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::*;

    #[test]
    fn it_works()
    {
        let a = [[1, -1], [2, -2], [3, -3]];
        let b = a.into_bulk()
            .flatten()
            .collect_array();

        println!("{b:?}")
    }
}