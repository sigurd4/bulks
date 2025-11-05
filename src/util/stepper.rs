use core::range::Step;

pub struct Stepper<U, const REV: bool = false>
where
    U: Step + Copy
{
    i: U
}
impl<U, const REV: bool> Stepper<U, REV>
where
    U: Step + Copy
{
    pub const fn new(i: U) -> Self
    {
        Self {
            i
        }
    }
}
impl<T, U, const REV: bool> FnOnce<(T,)> for Stepper<U, REV>
where
    U: Step + Copy
{
    type Output = (U, T);

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        (self.i, x)
    }
}
impl<T, U, const REV: bool> FnMut<(T,)> for Stepper<U, REV>
where
    U: Step + Copy
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        let i = self.i;
        if REV
        {
            self.i = U::forward(i, 1);
        }
        else
        {
            self.i = U::backward(i, 1);
        }
        (i, x)
    }
}