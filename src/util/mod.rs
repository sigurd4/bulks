moddef::moddef!(
    flat(pub) mod {
        array_buffer for cfg(feature = "array_chunks"),
        same,
        bulk_length,
        collect_length,
        length
    }
);