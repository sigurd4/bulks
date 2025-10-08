mod private
{
    pub trait Same<T>
    {
        fn same(self) -> Option<T>;
    }
    impl<T, U> Same<T> for U
    {
        default fn same(self) -> Option<T>
        {
            None
        }
    }
    impl<T> Same<T> for T
    {
        fn same(self) -> Option<T>
        {
            Some(self)
        }
    }
}

pub trait Same
{
    fn same<T>(self) -> Option<T>;
}
impl<U> Same for U
{
    fn same<T>(self) -> Option<T>
    {
        private::Same::<T>::same(self)
    }
}