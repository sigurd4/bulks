use core::mem::MaybeUninit;

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