pub trait RemoveRef
{
    type WithoutRef: ?Sized;
}

impl<T> RemoveRef for T
where
    T: ?Sized
{
    default type WithoutRef = T;
}
impl<T> RemoveRef for &T
where
    T: ?Sized
{
    type WithoutRef = <T as RemoveRef>::WithoutRef;
}
impl<T, const N: usize> RemoveRef for [T; N]
{
    type WithoutRef = Self;
}