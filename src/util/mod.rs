use core::{mem::{ManuallyDrop, MaybeUninit}};

moddef::moddef!(
    flat(pub) mod {
        array_buffer,
        const_step,
        guard,
        infinite_iterator,
        mutator,
        stepper,
        take_one,
        yield_once
    }
);

pub(crate) const fn split_array_ref<T, const N: usize, const M: usize>(array: &[T; N]) -> (&[T; N.min(M)], &[T; N.saturating_sub(M)])
{
    let ptr = array.as_ptr();
    unsafe {
        (ptr.cast::<[_; _]>().as_ref_unchecked(), ptr.add(N.min(M)).cast::<[_; _]>().as_ref_unchecked())
    }
}
pub(crate) const fn split_array_mut<T, const N: usize, const M: usize>(array: &mut [T; N]) -> (&mut [T; N.min(M)], &mut [T; N.saturating_sub(M)])
{
    let ptr = array.as_mut_ptr();
    unsafe {
        (ptr.cast::<[_; _]>().as_mut_unchecked(), ptr.add(N.min(M)).cast::<[_; _]>().as_mut_unchecked())
    }
}
pub(crate) const fn split_array<T, const N: usize, const M: usize>(array: [T; N]) -> ([T; N.min(M)], [T; N.saturating_sub(M)])
{
    #[repr(C)]
    struct Pair<L, R>
    {
        left: L,
        right: R
    }
    #[repr(C)]
    union Split<T, L, R>
    {
        concat: ManuallyDrop<T>,
        split: ManuallyDrop<Pair<L, R>>
    }

    let Pair { left, right } = unsafe {
        ManuallyDrop::into_inner(
            Split {
                concat: ManuallyDrop::new(array)
            }.split
        )
    };

    (left, right)
}

pub(crate) macro collect_array_with {
    (|$pusher:ident| $for_each:expr; for $bulk:ty) => {
        {
            use crate::StaticBulk;

            let mut array = MaybeUninit::<<$bulk as StaticBulk>::Array<<$bulk as IntoIterator>::Item>>::uninit();
            let array_mut = unsafe {
                array_trait::AsSlice::as_mut_slice(
                    array.as_mut_ptr().cast::<<$bulk as StaticBulk>::Array<MaybeUninit<<$bulk as IntoIterator>::Item>>>().as_mut().unwrap()
                )
            };
            let mut guard = Guard { array_mut, initialized: 0..0 };

            struct Closure<'a, 'b, T>
            {
                guard: &'a mut Guard<'b, T>
            }

            impl<'a, 'b, T> const FnOnce<(T,)> for Closure<'a, 'b, T>
            {
                type Output = ();

                extern "rust-call" fn call_once(mut self, args: (T,)) -> Self::Output
                {
                    self.call_mut(args)
                }
            }
            impl<'a, 'b, T> const FnMut<(T,)> for Closure<'a, 'b, T>
            {
                extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
                {
                    unsafe {
                        self.guard.push_back_unchecked(x);
                    }
                }
            }

            let $pusher = Closure {
                guard: &mut guard
            };

            $for_each;
            
            core::mem::forget(guard);
            unsafe {
                MaybeUninit::assume_init(array)
            }
        }
    }
}


pub(crate) macro try_collect_array_with {
    (|$pusher:ident| $try_for_each:expr; for $bulk:ty) => {
        {
            use core::ops::{Try, Residual};
            use crate::StaticBulk;

            let mut array = MaybeUninit::<<$bulk as StaticBulk>::Array<<<$bulk as IntoIterator>::Item as Try>::Output>>::uninit();
            let array_mut = unsafe {
                array_trait::AsSlice::as_mut_slice(
                    array.as_mut_ptr().cast::<<$bulk as StaticBulk>::Array<MaybeUninit<<<$bulk as IntoIterator>::Item as Try>::Output>>>().as_mut().unwrap()
                )
            };
            let mut guard = Guard { array_mut, initialized: 0..0 };

            struct Closure<'a, 'b, T>
            where
                T: Try<Residual: Residual<()>>
            {
                guard: &'a mut Guard<'b, <T as Try>::Output>
            }

            impl<'a, 'b, T> const FnOnce<(T,)> for Closure<'a, 'b, T>
            where
                T: ~const Try<Residual: Residual<(), TryType: ~const Try>>
            {
                type Output = <<T as Try>::Residual as Residual<()>>::TryType;

                extern "rust-call" fn call_once(self, (x,): (T,)) -> Self::Output
                {
                    unsafe {
                        self.guard.push_back_unchecked(x?);
                    }
                    Try::from_output(())
                }
            }
            impl<'a, 'b, T> const FnMut<(T,)> for Closure<'a, 'b, T>
            where
                T: ~const Try<Residual: Residual<(), TryType: ~const Try>>
            {
                extern "rust-call" fn call_mut(&mut self, (x,): (T,)) -> Self::Output
                {
                    unsafe {
                        self.guard.push_back_unchecked(x?);
                    }
                    Try::from_output(())
                }
            }

            let $pusher = Closure {
                guard: &mut guard
            };

            $try_for_each;
            
            core::mem::forget(guard);
            unsafe {
                MaybeUninit::assume_init(array)
            }
        }
    }
}