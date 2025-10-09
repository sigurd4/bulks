mod private
{
    pub trait Same<T>: Sized
    {
        fn same(self) -> Result<T, Self>;
    }
    impl<T, U> Same<T> for U
    {
        default fn same(self) -> Result<T, Self>
        {
            Err(self)
        }
    }
    impl<T> Same<T> for T
    {
        fn same(self) -> Result<T, Self>
        {
            Ok(self)
        }
    }
}

pub trait Same: Sized
{
    fn same<T>(self) -> Result<T, Self>;
}
impl<U> Same for U
{
    fn same<T>(self) -> Result<T, Self>
    {
        private::Same::<T>::same(self)
    }
}