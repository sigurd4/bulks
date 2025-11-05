use core::ops::RangeFrom;

/// # Safety
/// 
/// Iterator must yield an infinite amount of elements
pub unsafe trait InfiniteIterator: Iterator
{

}

unsafe impl<I, const N: usize> InfiniteIterator for core::iter::ArrayChunks<I, N>
where
    I: InfiniteIterator
{
    
}
unsafe impl<A, B> InfiniteIterator for core::iter::Chain<A, B>
where
    (A, B): private::InfiniteIteratorPairSpec<EitherInfiniteIterator: InfiniteIterator>,
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::Cloned<I>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::Copied<I>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::Cycle<I>
where
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::Enumerate<I>
where
    I: InfiniteIterator
{
    
}
unsafe impl<I, P> InfiniteIterator for core::iter::Filter<I, P>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I, F> InfiniteIterator for core::iter::FilterMap<I, F>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I, U, V, F> InfiniteIterator for core::iter::FlatMap<I, U, F>
where
    U: IntoIterator<IntoIter = V>,
    (I, V): private::InfiniteIteratorPairSpec<EitherInfiniteIterator: InfiniteIterator>,
    Self: Iterator
{
    
}
unsafe impl<I, V> InfiniteIterator for core::iter::Flatten<I>
where
    I: Iterator<Item: IntoIterator<IntoIter = V>>,
    (I, V): private::InfiniteIteratorPairSpec<EitherInfiniteIterator: InfiniteIterator>,
    Self: Iterator
{
    
}
unsafe impl<I, F> InfiniteIterator for core::iter::Inspect<I, F>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::Intersperse<I>
where
    I: InfiniteIterator<Item: Clone>
{
    
}
unsafe impl<I, G> InfiniteIterator for core::iter::IntersperseWith<I, G>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I, F> InfiniteIterator for core::iter::Map<I, F>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I, F, const N: usize> InfiniteIterator for core::iter::MapWindows<I, F, N>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::Peekable<I>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<A> InfiniteIterator for core::iter::Repeat<A>
where
    Self: Iterator
{
    
}
unsafe impl<F> InfiniteIterator for core::iter::RepeatWith<F>
where
    Self: Iterator
{
    
}
unsafe impl<I, St, F> InfiniteIterator for core::iter::Scan<I, St, F>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::Skip<I>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I, P> InfiniteIterator for core::iter::SkipWhile<I, P>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<I> InfiniteIterator for core::iter::StepBy<I>
where
    I: InfiniteIterator,
    Self: Iterator
{
    
}
unsafe impl<A, B> InfiniteIterator for core::iter::Zip<A, B>
where
    (A, B): private::InfiniteIteratorPairSpec<BothInfiniteIterator: InfiniteIterator>,
    A: InfiniteIterator,
    B: InfiniteIterator,
    Self: Iterator
{
    
}

unsafe impl<Idx> InfiniteIterator for RangeFrom<Idx>
where
    Self: Iterator
{
    
}

mod private
{
    use core::marker::Tuple;

    use crate::util::InfiniteIterator;

    pub trait InfiniteIteratorPairSpec: Tuple
    {
        type EitherInfiniteIterator: Iterator;
        type BothInfiniteIterator: Iterator;
    }

    impl<A, B> InfiniteIteratorPairSpec for (A, B)
    where
        A: Iterator,
        B: Iterator
    {
        default type EitherInfiniteIterator = A;
        default type BothInfiniteIterator = B;
    }
    impl<A, B> InfiniteIteratorPairSpec for (A, B)
    where
        A: Iterator,
        B: InfiniteIterator
    {
        type EitherInfiniteIterator = B;
        type BothInfiniteIterator = A;
    }

}