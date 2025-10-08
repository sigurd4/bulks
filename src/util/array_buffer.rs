use core::mem::MaybeUninit;

pub struct ArrayBuffer<T, const N: usize>
{
    data: [MaybeUninit<T>; N],
    len: usize
}

impl<T, const N: usize> ArrayBuffer<T, N>
{
    pub const fn new() -> Self
    {
        Self {
            data: [const {MaybeUninit::uninit()}; N],
            len: 0
        }
    }

    pub const fn push(&mut self, value: T)
    {
        let dst = self.data.get_mut(self.len)
            .expect("Exceeded array buffer capacity");
        dst.write(value);
        self.len += 1;
    }
}

impl<T, const N: usize> Extend<T> for ArrayBuffer<T, N>
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I)
    {
        for item in iter
        {
            self.push(item);
        }
    }

    fn extend_one(&mut self, item: T)
    {
        self.push(item);
    }
}