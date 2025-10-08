use array_trait::Array;

use crate::Bulk;

pub trait StaticBulk: Bulk
{
    type Array: Array<Elem = Self::Item>;

    fn collect_array(self) -> Self::Array;
}