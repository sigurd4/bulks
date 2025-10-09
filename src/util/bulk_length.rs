use core::ptr::Pointee;

use crate::{util::{Length, Same}, Bulk, StaticBulk};

pub trait BulkLength: Bulk
{
    type Length: Length<Elem = <Self as IntoIterator>::Item> + ?Sized;

    fn len_metadata(&self) -> <Self::Length as Pointee>::Metadata;
}
impl<T> BulkLength for T
where
    Self: Bulk
{
    default type Length = [Self::Item];

    default fn len_metadata(&self) -> <Self::Length as Pointee>::Metadata
    {
        self.len().same().ok().unwrap()
    }
}
impl<T> BulkLength for T
where
    Self: StaticBulk
{
    type Length = <Self as StaticBulk>::Array;

    fn len_metadata(&self) -> <Self::Length as Pointee>::Metadata
    {
        
    }
}