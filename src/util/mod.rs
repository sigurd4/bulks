use core::mem::MaybeUninit;

use crate::Guard;

moddef::moddef!(
    flat(pub) mod {
        array_buffer,
        same,
        bulk_length,
        collect_length,
        infinite_iterator,
        length,
        mutator,
    }
);

pub(crate) const fn new_init_array<T, const N: usize>(array: [T; N]) -> [MaybeUninit<T>; N]
{
    unsafe {
        core::intrinsics::transmute_unchecked(array)
    }
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