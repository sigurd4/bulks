trait Bulk
{
    
}

trait DoubleEndedBulk: Bulk
{

}

trait InplaceBulk: Bulk
{
    type EachRef<'a>: InplaceBulk<EachRef<'a> = Self::EachRef<'a>> + 'a
    where
        Self: 'a;
}

struct Rev<I>
where
    I: DoubleEndedBulk
{
    bulk: I
}
impl<I> Bulk for Rev<I>
where
    I: DoubleEndedBulk
{
    
}
impl<I> InplaceBulk for Rev<I>
where
    I: DoubleEndedBulk + for<'a, 'b> InplaceBulk<EachRef<'a>: InplaceBulk<EachRef<'b> = I::EachRef<'b>> + DoubleEndedBulk + 'b>
{
    type EachRef<'a> = Rev<I::EachRef<'a>>
    where
        Self: 'a;
}