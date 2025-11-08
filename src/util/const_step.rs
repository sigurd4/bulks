use core::{ascii::Char as AsciiChar, net::{Ipv4Addr, Ipv6Addr}, range::Step};

/// Temporary solution because they haven't made the `Step` trait const yet... :(
pub const trait ConstStep: Step
{
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>);

    fn forward_checked(start: Self, count: usize) -> Option<Self>;

    fn forward(start: Self, count: usize) -> Self
    {
        ConstStep::forward_checked(start, count).expect("overflow in `Step::forward`")
    }

    unsafe fn forward_unchecked(start: Self, count: usize) -> Self
    {
        ConstStep::forward(start, count)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self>;

    fn backward(start: Self, count: usize) -> Self
    {
        ConstStep::backward_checked(start, count).expect("overflow in `Step::backward`")
    }

    unsafe fn backward_unchecked(start: Self, count: usize) -> Self
    {
        ConstStep::backward(start, count)
    }
}
impl<T> ConstStep for T
where
    T: Step
{
    default fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>)
    {
        Step::steps_between(start, end)
    }

    default fn forward_checked(start: Self, count: usize) -> Option<Self>
    {
        Step::forward_checked(start, count)
    }

    default fn forward(start: Self, count: usize) -> Self
    {
        Step::forward(start, count)
    }

    default unsafe fn forward_unchecked(start: Self, count: usize) -> Self
    {
        unsafe {
            Step::forward_unchecked(start, count)
        }
    }

    default fn backward_checked(start: Self, count: usize) -> Option<Self>
    {
        Step::backward_checked(start, count)
    }

    default fn backward(start: Self, count: usize) -> Self
    {
        Step::backward(start, count)
    }

    default unsafe fn backward_unchecked(start: Self, count: usize) -> Self
    {
        unsafe {
            Step::backward_unchecked(start, count)
        }
    }
}

// Separate impls for signed ranges because the distance within a signed range can be larger
// than the signed::MAX value. Therefore `as` casting to the signed type would be incorrect.
macro_rules! step_signed_methods {
    ($unsigned: ty) => {
        #[inline]
        unsafe fn forward_unchecked(start: Self, n: usize) -> Self {
            // SAFETY: the caller has to guarantee that `start + n` doesn't overflow.
            unsafe { start.checked_add_unsigned(n as $unsigned).unwrap_unchecked() }
        }

        #[inline]
        unsafe fn backward_unchecked(start: Self, n: usize) -> Self {
            // SAFETY: the caller has to guarantee that `start - n` doesn't overflow.
            unsafe { start.checked_sub_unsigned(n as $unsigned).unwrap_unchecked() }
        }
    };
}

macro_rules! step_unsigned_methods {
    () => {
        #[inline]
        unsafe fn forward_unchecked(start: Self, n: usize) -> Self {
            // SAFETY: the caller has to guarantee that `start + n` doesn't overflow.
            unsafe { start.unchecked_add(n as Self) }
        }

        #[inline]
        unsafe fn backward_unchecked(start: Self, n: usize) -> Self {
            // SAFETY: the caller has to guarantee that `start - n` doesn't overflow.
            unsafe { start.unchecked_sub(n as Self) }
        }
    };
}

// These are still macro-generated because the integer literals resolve to different types.
macro_rules! step_identical_methods {
    () => {
        #[inline]
        #[allow(arithmetic_overflow)]
        #[rustc_inherit_overflow_checks]
        fn forward(start: Self, n: usize) -> Self {
            // In debug builds, trigger a panic on overflow.
            // This should optimize completely out in release builds.
            if ConstStep::forward_checked(start, n).is_none() {
                let _ = Self::MAX + 1;
            }
            // Do wrapping math to allow e.g. `Step::forward(-128i8, 255)`.
            start.wrapping_add(n as Self)
        }

        #[inline]
        #[allow(arithmetic_overflow)]
        #[rustc_inherit_overflow_checks]
        fn backward(start: Self, n: usize) -> Self {
            // In debug builds, trigger a panic on overflow.
            // This should optimize completely out in release builds.
            if ConstStep::backward_checked(start, n).is_none() {
                let _ = Self::MIN - 1;
            }
            // Do wrapping math to allow e.g. `Step::backward(127i8, 255)`.
            start.wrapping_sub(n as Self)
        }
    };
}

macro_rules! step_integer_impls {
    {
        narrower than or same width as usize:
            $( [ $u_narrower:ident $i_narrower:ident ] ),+;
        wider than usize:
            $( [ $u_wider:ident $i_wider:ident ] ),+;
    } => {
        $(
            #[allow(unreachable_patterns)]
            impl const ConstStep for $u_narrower {
                step_identical_methods!();
                step_unsigned_methods!();

                #[inline]
                fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                    if *start <= *end {
                        // This relies on $u_narrower <= usize
                        let steps = (*end - *start) as usize;
                        (steps, Some(steps))
                    } else {
                        (0, None)
                    }
                }

                #[inline]
                fn forward_checked(start: Self, n: usize) -> Option<Self> {
                    match Self::try_from(n) {
                        Ok(n) => start.checked_add(n),
                        Err(_) => None, // if n is out of range, `unsigned_start + n` is too
                    }
                }

                #[inline]
                fn backward_checked(start: Self, n: usize) -> Option<Self> {
                    match Self::try_from(n) {
                        Ok(n) => start.checked_sub(n),
                        Err(_) => None, // if n is out of range, `unsigned_start - n` is too
                    }
                }
            }

            #[allow(unreachable_patterns)]
            impl const ConstStep for $i_narrower {
                step_identical_methods!();
                step_signed_methods!($u_narrower);

                #[inline]
                fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                    if *start <= *end {
                        // This relies on $i_narrower <= usize
                        //
                        // Casting to isize extends the width but preserves the sign.
                        // Use wrapping_sub in isize space and cast to usize to compute
                        // the difference that might not fit inside the range of isize.
                        let steps = (*end as isize).wrapping_sub(*start as isize) as usize;
                        (steps, Some(steps))
                    } else {
                        (0, None)
                    }
                }

                #[inline]
                fn forward_checked(start: Self, n: usize) -> Option<Self> {
                    match $u_narrower::try_from(n) {
                        Ok(n) => {
                            // Wrapping handles cases like
                            // `Step::forward(-120_i8, 200) == Some(80_i8)`,
                            // even though 200 is out of range for i8.
                            let wrapped = start.wrapping_add(n as Self);
                            if wrapped >= start {
                                Some(wrapped)
                            } else {
                                None // Addition overflowed
                            }
                        }
                        // If n is out of range of e.g. u8,
                        // then it is bigger than the entire range for i8 is wide
                        // so `any_i8 + n` necessarily overflows i8.
                        Err(_) => None,
                    }
                }

                #[inline]
                fn backward_checked(start: Self, n: usize) -> Option<Self> {
                    match $u_narrower::try_from(n) {
                        Ok(n) => {
                            // Wrapping handles cases like
                            // `Step::forward(-120_i8, 200) == Some(80_i8)`,
                            // even though 200 is out of range for i8.
                            let wrapped = start.wrapping_sub(n as Self);
                            if wrapped <= start {
                                Some(wrapped)
                            } else {
                                None // Subtraction overflowed
                            }
                        }
                        // If n is out of range of e.g. u8,
                        // then it is bigger than the entire range for i8 is wide
                        // so `any_i8 - n` necessarily overflows i8.
                        Err(_) => None,
                    }
                }
            }
        )+

        $(
            #[allow(unreachable_patterns)]
            impl const ConstStep for $u_wider {
                step_identical_methods!();
                step_unsigned_methods!();

                #[inline]
                fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                    if *start <= *end {
                        if let Ok(steps) = usize::try_from(*end - *start) {
                            (steps, Some(steps))
                        } else {
                            (usize::MAX, None)
                        }
                    } else {
                        (0, None)
                    }
                }

                #[inline]
                fn forward_checked(start: Self, n: usize) -> Option<Self> {
                    start.checked_add(n as Self)
                }

                #[inline]
                fn backward_checked(start: Self, n: usize) -> Option<Self> {
                    start.checked_sub(n as Self)
                }
            }

            #[allow(unreachable_patterns)]
            impl const ConstStep for $i_wider {
                step_identical_methods!();
                step_signed_methods!($u_wider);

                #[inline]
                fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
                    if *start <= *end {
                        match end.checked_sub(*start) {
                            Some(result) => {
                                if let Ok(steps) = usize::try_from(result) {
                                    (steps, Some(steps))
                                } else {
                                    (usize::MAX, None)
                                }
                            }
                            // If the difference is too big for e.g. i128,
                            // it's also gonna be too big for usize with fewer bits.
                            None => (usize::MAX, None),
                        }
                    } else {
                        (0, None)
                    }
                }

                #[inline]
                fn forward_checked(start: Self, n: usize) -> Option<Self> {
                    start.checked_add(n as Self)
                }

                #[inline]
                fn backward_checked(start: Self, n: usize) -> Option<Self> {
                    start.checked_sub(n as Self)
                }
            }
        )+
    };
}

#[cfg(target_pointer_width = "64")]
step_integer_impls! {
    narrower than or same width as usize: [u8 i8], [u16 i16], [u32 i32], [u64 i64], [usize isize];
    wider than usize: [u128 i128];
}

#[cfg(target_pointer_width = "32")]
step_integer_impls! {
    narrower than or same width as usize: [u8 i8], [u16 i16], [u32 i32], [usize isize];
    wider than usize: [u64 i64], [u128 i128];
}

#[cfg(target_pointer_width = "16")]
step_integer_impls! {
    narrower than or same width as usize: [u8 i8], [u16 i16], [usize isize];
    wider than usize: [u32 i32], [u64 i64], [u128 i128];
}

impl const ConstStep for char {
    #[inline]
    fn steps_between(&start: &char, &end: &char) -> (usize, Option<usize>) {
        let start = start as u32;
        let end = end as u32;
        if start <= end {
            let count = end - start;
            if start < 0xD800 && 0xE000 <= end {
                if let Ok(steps) = usize::try_from(count - 0x800) {
                    (steps, Some(steps))
                } else {
                    (usize::MAX, None)
                }
            } else {
                if let Ok(steps) = usize::try_from(count) {
                    (steps, Some(steps))
                } else {
                    (usize::MAX, None)
                }
            }
        } else {
            (0, None)
        }
    }

    #[inline]
    fn forward_checked(start: char, count: usize) -> Option<char> {
        let start = start as u32;
        let mut res = ConstStep::forward_checked(start, count)?;
        if start < 0xD800 && 0xD800 <= res {
            res = ConstStep::forward_checked(res, 0x800)?;
        }
        if res <= char::MAX as u32 {
            // SAFETY: res is a valid unicode scalar
            // (below 0x110000 and not in 0xD800..0xE000)
            Some(unsafe { char::from_u32_unchecked(res) })
        } else {
            None
        }
    }

    #[inline]
    fn backward_checked(start: char, count: usize) -> Option<char> {
        let start = start as u32;
        let mut res = ConstStep::backward_checked(start, count)?;
        if start >= 0xE000 && 0xE000 > res {
            res = ConstStep::backward_checked(res, 0x800)?;
        }
        // SAFETY: res is a valid unicode scalar
        // (below 0x110000 and not in 0xD800..0xE000)
        Some(unsafe { char::from_u32_unchecked(res) })
    }

    #[inline]
    unsafe fn forward_unchecked(start: char, count: usize) -> char {
        let start = start as u32;
        // SAFETY: the caller must guarantee that this doesn't overflow
        // the range of values for a char.
        let mut res = unsafe { ConstStep::forward_unchecked(start, count) };
        if start < 0xD800 && 0xD800 <= res {
            // SAFETY: the caller must guarantee that this doesn't overflow
            // the range of values for a char.
            res = unsafe { ConstStep::forward_unchecked(res, 0x800) };
        }
        // SAFETY: because of the previous contract, this is guaranteed
        // by the caller to be a valid char.
        unsafe { char::from_u32_unchecked(res) }
    }

    #[inline]
    unsafe fn backward_unchecked(start: char, count: usize) -> char {
        let start = start as u32;
        // SAFETY: the caller must guarantee that this doesn't overflow
        // the range of values for a char.
        let mut res = unsafe { ConstStep::backward_unchecked(start, count) };
        if start >= 0xE000 && 0xE000 > res {
            // SAFETY: the caller must guarantee that this doesn't overflow
            // the range of values for a char.
            res = unsafe { ConstStep::backward_unchecked(res, 0x800) };
        }
        // SAFETY: because of the previous contract, this is guaranteed
        // by the caller to be a valid char.
        unsafe { char::from_u32_unchecked(res) }
    }
}

impl const ConstStep for AsciiChar {
    #[inline]
    fn steps_between(&start: &AsciiChar, &end: &AsciiChar) -> (usize, Option<usize>) {
        ConstStep::steps_between(&start.to_u8(), &end.to_u8())
    }

    #[inline]
    fn forward_checked(start: AsciiChar, count: usize) -> Option<AsciiChar> {
        let end = ConstStep::forward_checked(start.to_u8(), count)?;
        AsciiChar::from_u8(end)
    }

    #[inline]
    fn backward_checked(start: AsciiChar, count: usize) -> Option<AsciiChar> {
        let end = ConstStep::backward_checked(start.to_u8(), count)?;

        // SAFETY: Values below that of a valid ASCII character are also valid ASCII
        Some(unsafe { AsciiChar::from_u8_unchecked(end) })
    }

    #[inline]
    unsafe fn forward_unchecked(start: AsciiChar, count: usize) -> AsciiChar {
        // SAFETY: Caller asserts that result is a valid ASCII character,
        // and therefore it is a valid u8.
        let end = unsafe { ConstStep::forward_unchecked(start.to_u8(), count) };

        // SAFETY: Caller asserts that result is a valid ASCII character.
        unsafe { AsciiChar::from_u8_unchecked(end) }
    }

    #[inline]
    unsafe fn backward_unchecked(start: AsciiChar, count: usize) -> AsciiChar {
        // SAFETY: Caller asserts that result is a valid ASCII character,
        // and therefore it is a valid u8.
        let end = unsafe { ConstStep::backward_unchecked(start.to_u8(), count) };

        // SAFETY: Caller asserts that result is a valid ASCII character.
        unsafe { AsciiChar::from_u8_unchecked(end) }
    }
}

impl const ConstStep for Ipv4Addr {
    #[inline]
    fn steps_between(&start: &Ipv4Addr, &end: &Ipv4Addr) -> (usize, Option<usize>) {
        ConstStep::steps_between(&start.to_bits(), &end.to_bits())
    }

    #[inline]
    fn forward_checked(start: Ipv4Addr, count: usize) -> Option<Ipv4Addr> {
        ConstStep::forward_checked(start.to_bits(), count).map(Ipv4Addr::from_bits)
    }

    #[inline]
    fn backward_checked(start: Ipv4Addr, count: usize) -> Option<Ipv4Addr> {
        ConstStep::backward_checked(start.to_bits(), count).map(Ipv4Addr::from_bits)
    }

    #[inline]
    unsafe fn forward_unchecked(start: Ipv4Addr, count: usize) -> Ipv4Addr {
        // SAFETY: Since u32 and Ipv4Addr are losslessly convertible,
        //   this is as safe as the u32 version.
        Ipv4Addr::from_bits(unsafe { ConstStep::forward_unchecked(start.to_bits(), count) })
    }

    #[inline]
    unsafe fn backward_unchecked(start: Ipv4Addr, count: usize) -> Ipv4Addr {
        // SAFETY: Since u32 and Ipv4Addr are losslessly convertible,
        //   this is as safe as the u32 version.
        Ipv4Addr::from_bits(unsafe { ConstStep::backward_unchecked(start.to_bits(), count) })
    }
}

impl const ConstStep for Ipv6Addr {
    #[inline]
    fn steps_between(&start: &Ipv6Addr, &end: &Ipv6Addr) -> (usize, Option<usize>) {
        ConstStep::steps_between(&start.to_bits(), &end.to_bits())
    }

    #[inline]
    fn forward_checked(start: Ipv6Addr, count: usize) -> Option<Ipv6Addr> {
        ConstStep::forward_checked(start.to_bits(), count).map(Ipv6Addr::from_bits)
    }

    #[inline]
    fn backward_checked(start: Ipv6Addr, count: usize) -> Option<Ipv6Addr> {
        ConstStep::backward_checked(start.to_bits(), count).map(Ipv6Addr::from_bits)
    }

    #[inline]
    unsafe fn forward_unchecked(start: Ipv6Addr, count: usize) -> Ipv6Addr {
        // SAFETY: Since u128 and Ipv6Addr are losslessly convertible,
        //   this is as safe as the u128 version.
        Ipv6Addr::from_bits(unsafe { ConstStep::forward_unchecked(start.to_bits(), count) })
    }

    #[inline]
    unsafe fn backward_unchecked(start: Ipv6Addr, count: usize) -> Ipv6Addr {
        // SAFETY: Since u128 and Ipv6Addr are losslessly convertible,
        //   this is as safe as the u128 version.
        Ipv6Addr::from_bits(unsafe { ConstStep::backward_unchecked(start.to_bits(), count) })
    }
}