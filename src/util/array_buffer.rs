use core::mem::MaybeUninit;

pub struct ArrayBuffer<T, const N: usize, const REV: bool>
{
    data: [MaybeUninit<T>; N],
    len: usize
}

impl<T, const N: usize, const REV: bool> ArrayBuffer<T, N, REV>
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
        let i = if !REV
        {
            let i = self.len;
            self.len += 1;
            i
        }
        else
        {
            self.len += 1;
            N.checked_sub(self.len).expect("Exceeded array buffer capacity")
        };
        let dst = self.data.get_mut(i)
            .expect("Exceeded array buffer capacity");
        dst.write(value);
    }

    pub const fn push_out_whole(&mut self, value: T) -> Option<[T; N]>
    {
        let array = self.take_array();
        self.push(value);
        array
    }

    pub const fn push_take_array(&mut self, value: T) -> Option<[T; N]>
    {
        match self.push_out_whole(value)
        {
            Some(array) => Some(array),
            None => self.take_array()
        }
    }

    pub const fn take_array(&mut self) -> Option<[T; N]>
    {
        match self.as_array()
        {
            Some(array) => {
                let array = unsafe {
                    core::ptr::read(array)
                };
                self.len = 0;
                Some(array)
            },
            None => None
        }
    }

    pub const fn push_out(&mut self, value: T) -> Option<T>
    {
        if N == 0
        {
            return Some(value)
        }
        if let Some(array) = self.as_mut_array()
        {
            let ptr = array.as_mut_ptr();
            if N == 1
            {
                return unsafe {
                    Some(core::ptr::replace(ptr, value))
                }
            }
            let out = unsafe {
                ptr.read()
            };
            if !REV
            {
                unsafe {
                    ptr.add(1).copy_to(ptr, N - 1);
                    ptr.add(N - 1).write(value);
                }
            }
            else
            {
                unsafe {
                    ptr.copy_to(ptr.add(1), N - 1);
                    ptr.write(value);
                }
            }
            Some(out)
        }
        else
        {
            self.push(value);
            None
        }
    }

    pub const fn as_mut_array(&mut self) -> Option<&mut [T; N]>
    {
        if self.len >= N
        {
            return Some(unsafe {self.data.assume_init_mut().as_mut_array().unwrap_unchecked()})
        }
        None
    }

    pub const fn as_array(&self) -> Option<&[T; N]>
    {
        if self.len >= N
        {
            return Some(unsafe {self.data.assume_init_ref().as_array().unwrap_unchecked()})
        }
        None
    }
}

impl<T, const N: usize, const REV: bool> Extend<T> for ArrayBuffer<T, N, REV>
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