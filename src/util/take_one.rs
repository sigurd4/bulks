use core::marker::Tuple;

#[derive(Clone, Copy)]
pub struct TakeOne<F>
{
    f: Option<F>
}

impl<F> TakeOne<F>
{
    pub const fn new(f: F) -> Self
    {
        Self {
            f: Some(f)
        }
    }
}

impl<F, Args> const FnOnce<Args> for TakeOne<F>
where
    F: ~const FnOnce<Args>,
    Args: Tuple
{
    type Output = F::Output;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output
    {
        let Self { f } = self;
        f.expect("Can only be called once")
            .call_once(args)
    }
}
impl<F, Args> const FnMut<Args> for TakeOne<F>
where
    F: ~const FnOnce<Args>,
    Args: Tuple
{
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output
    {
        let Self { f } = self;
        Self { f: f.take() }.call_once(args)
    }
}