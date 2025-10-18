pub trait BulkableIterator: private::BulkableIteratorSpec<_Length = usize>
{
    fn bulk_len(&self) -> usize;
}

impl<I> BulkableIterator for I
where
    I: private::BulkableIteratorSpec<_Length = usize>
{
    fn bulk_len(&self) -> usize
    {
        self._len()
    }
}

mod private
{
    use crate::{util::Same, BulkableIterator};

    pub trait BulkableIteratorSpecSpec: Iterator
    {
        type __Length;

        fn __len(&self) -> Self::__Length;
    }

    impl<I> BulkableIteratorSpecSpec for I
    where
        I: Iterator
    {
        default type __Length = ();

        default fn __len(&self) -> Self::__Length
        {
            ().same().ok().unwrap()
        }
    }

    impl<I> BulkableIteratorSpecSpec for I
    where
        I: ExactSizeIterator
    {
        type __Length = usize;

        fn __len(&self) -> Self::__Length
        {
            self.len()
        }
    }

    pub trait BulkableIteratorSpec: Iterator
    {
        type _Length;

        fn _len(&self) -> Self::_Length;
    }

    impl<I> BulkableIteratorSpec for I
    where
        I: Iterator
    {
        default type _Length = <Self as BulkableIteratorSpecSpec>::__Length;

        default fn _len(&self) -> Self::_Length
        {
            self.__len().same().ok().unwrap()
        }
    }
    impl<A, B> BulkableIteratorSpec for core::iter::Chain<A, B>
    where
        A: BulkableIterator,
        B: BulkableIterator<Item = A::Item>
    {
        type _Length = usize;

        fn _len(&self) -> Self::_Length
        {
            let size = self.size_hint();
            size.1.unwrap_or(size.0)
        }
    }
}