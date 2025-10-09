use crate::{util::Same, Bulk, IntoBulk};

pub mod iter
{
    pub struct Bulk<T>
    where
        T: IntoIterator<IntoIter: ExactSizeIterator>
    {
        pub(super) inner: T::IntoIter
    }
}

impl<T, A, I> IntoIterator for iter::Bulk<T>
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A>
{
    type IntoIter = I;
    type Item = A;
    
    fn into_iter(self) -> Self::IntoIter
    {
        self.inner.into_iter()
    }
}

impl<T, A, I> IntoBulk for T
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A>
{
    default type IntoBulk = iter::Bulk<T>;

    default fn into_bulk(self) -> Self::IntoBulk
    {
        iter::Bulk::<T> {
            inner: self.into_iter()
        }.same().ok().unwrap()
    }
}

impl<T, A, I> Bulk for iter::Bulk<T>
where
    T: IntoIterator<Item = A, IntoIter = I>,
    I: ExactSizeIterator<Item = A>
{
    #[inline]
    fn len(&self) -> usize
    {
        self.inner.len()
    }

    #[inline]
    fn is_empty(&self) -> bool
    {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod test
{
    use crate::{Bulk, IntoBulk};

    #[test]
    fn vec()
    {
        let a = vec![1i32, 2, 3, 4, 5];
        let bulk = a.into_bulk().map(|x| x as f64);
        let b = bulk.collect::<Vec<f64>>();
        println!("{b:?}")
    }
}