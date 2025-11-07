use array_trait::length;

pub trait CollectLength<A>: Sized
{
    type Length: length::Length<Elem = A> + ?Sized;
}
impl<T, A> CollectLength<A> for T
{
    default type Length = [A];
}
impl<T, A, const N: usize> CollectLength<A> for [T; N]
{
    type Length = [A; N];
}