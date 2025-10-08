use array_trait::AsSlice;

mod private
{
    use array_trait::AsSlice;

    pub trait Length: IntoIterator + Sized
    {
        type Length: AsSlice<Elem = Self::Item> + ?Sized;
    }

    impl<T> Length for T
    where
        T: IntoIterator
    {
        default type Length = [T::Item];
    }
    impl<T, const N: usize> Length for [T; N]
    {
        type Length = [T; N];
    }
}

pub trait Length: IntoIterator + Sized
{
    type Length: AsSlice<Elem = Self::Item> + ?Sized;
}
impl<T> Length for T
where
    T: IntoIterator
{
    type Length = <T as private::Length>::Length;
}