moddef::moddef!(
    flat(pub) mod {
        array_chunks for cfg(feature = "array_chunks"),
        cloned,
        copied,
        empty,
        inspect,
        map,
        once_with,
        once,
        repeat_n,
        rev,
        zip
    }
);