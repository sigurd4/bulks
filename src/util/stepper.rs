use crate::Step;

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
impl<T, U, const REV: bool> const FnOnce<(T,)> for Stepper<U, REV>
where
    U: Step + Copy
{
    type Output = (U, T);

    extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
    {
        (self.i, x)
    }
}
impl<T, U, const REV: bool> const FnMut<(T,)> for Stepper<U, REV>
where
    U: ~const Step + Copy
{
    extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
    {
        if REV
        {
            self.i = Step::backward(self.i, 1);
        }
        let i = self.i;
        if !REV
        {
            self.i = Step::forward(self.i, 1);
        }
        (i, x)
    }
}