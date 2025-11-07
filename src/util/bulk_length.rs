use core::ptr::Pointee;

use array_trait::{same::Same, length};

use crate::{Bulk, StaticBulk};

pub trait BulkLength: Bulk
{
    type Length: length::Length<Elem = <Self as IntoIterator>::Item> + ?Sized;

    fn len_metadata(&self) -> <Self::Length as Pointee>::Metadata;
}
impl<T> BulkLength for T
where
    T: Bulk + ?Sized
{
    default type Length = [Self::Item];

    default fn len_metadata(&self) -> <Self::Length as Pointee>::Metadata
    {
        self.len().same().ok().unwrap()
    }
}
impl<T> BulkLength for T
where
    T: StaticBulk
{
    type Length = <Self as StaticBulk>::Array<Self::Item>;

    fn len_metadata(&self) -> <Self::Length as Pointee>::Metadata
    {
        
    }
}