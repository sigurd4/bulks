pub struct YieldOnce<T>
{
    x: T
}

impl<T> YieldOnce<T>
{
    pub const fn new(x: T) -> Self
    {
        Self {
            x
        }
    }
}
impl<T> FnOnce<()> for YieldOnce<T>
{
    type Output = T;

    extern "rust-call" fn call_once(self, (): ()) -> Self::Output
    {
        let Self { x } = self;
        x
    }
}
impl<T> FnMut<()> for YieldOnce<T>
where
    T: Clone
{
    extern "rust-call" fn call_mut(&mut self, (): ()) -> Self::Output
    {
        self.call(())
    }
}
impl<T> Fn<()> for YieldOnce<T>
where
    T: Clone
{
    extern "rust-call" fn call(&self, (): ()) -> Self::Output
    {
        let Self { x } = self;
        x.clone()
    }
}