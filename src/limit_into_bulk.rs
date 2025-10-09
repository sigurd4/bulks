use crate::{util::{BulkLength, Same}, Bulk, IntoBulk, Take};

pub trait LimitIntoBulk<B>: IntoIterator
where
    B: Bulk
{
    type LimitIntoBulk;

    fn limit_into_bulk(self, limit: &B) -> Self::LimitIntoBulk;
}

impl<T, B> LimitIntoBulk<B> for T
where
    T: IntoIterator,
    B: BulkLength
{
    default type LimitIntoBulk = Take<Infinite<T>, B::Length>;

    default fn limit_into_bulk(self, limit: &B) -> Self::LimitIntoBulk
    {
        Take::<T, B::Length>::new(self, limit.len_metadata()).same().ok().unwrap()
    }
}
impl<T, B> LimitIntoBulk<B> for T
where
    T: IntoBulk,
    B: Bulk
{
    type LimitIntoBulk = T;

    fn limit_into_bulk(self, _limit: &B) -> Self::LimitIntoBulk
    {
        self
    }
}