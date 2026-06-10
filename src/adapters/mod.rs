use array_trait::length::{self, LengthValue};

use crate::range::{Range, RangeInclusive};

moddef::moddef!(
    mod {
        array_chunks_with_remainder
    },
    flat(pub) mod {
        array_chunks,
        chain,
        cloned,
        copied,
        empty,
        enumerate_from,
        enumerate,
        flat_map,
        flatten,
        contained,
        inspect,
        intersperse_with,
        intersperse,
        map_windows,
        map,
        mutate,
        once_with,
        once,
        merge,
        repeat_n_with,
        repeat_n,
        resize_with,
        resize,
        rev,
        skip,
        step_by,
        take,
        zip
    }
);

pub const fn range<S, E>(start: S, end: E) -> Range<length::value::Length<S, ()>, length::value::Length<E, ()>>
where
    S: LengthValue,
    E: LengthValue
{
    Range::new(start, end)
}

pub const fn range_inclusive<S, E>(start: S, end: E) -> RangeInclusive<length::value::Length<S, ()>, length::value::Length<E, ()>>
where
    S: LengthValue,
    E: LengthValue
{
    RangeInclusive::new(start, end)
}