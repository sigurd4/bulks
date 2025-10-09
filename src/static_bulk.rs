use array_trait::Array;

use crate::{util::Length, Bulk};

pub trait StaticBulk: Bulk
{
    type Array: Array<Elem = Self::Item> + Length;

    fn collect_array(self) -> Self::Array;
}