use array_trait::AsSlice;

use crate::{Bulk, StaticBulk};

pub trait BulkLength: Bulk
{
    type Length: AsSlice<Elem = Self::Item> + ?Sized;
}
impl<T> BulkLength for T
where
    Self: Bulk
{
    default type Length = [Self::Item];
}
impl<T> BulkLength for T
where
    Self: StaticBulk
{
    type Length = <Self as StaticBulk>::Array;
}