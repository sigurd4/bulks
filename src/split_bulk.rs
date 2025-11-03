use crate::{Bulk, util::LengthSpec};

pub const trait SplitBulk<L>: Bulk
where
    L: LengthSpec
{
    type Left: Bulk<Item = Self::Item>;
    type Right: Bulk<Item = Self::Item>;

    #[track_caller]
    fn saturating_split_at(self, n: L) -> (Self::Left, Self::Right)
    where
        Self: Sized;
}