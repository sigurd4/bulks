moddef::moddef!(
    flat(pub) mod {
        array_chunks for cfg(feature = "array_chunks"),
        map,
        cloned,
        inspect,
        copied,
        rev,
        zip for cfg(disabled),
    }
);