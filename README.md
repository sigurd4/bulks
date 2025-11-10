[![Build Status (nightly)](https://github.com/sigurd4/bulks/workflows/Build-nightly/badge.svg)](https://github.com/sigurd4/bulks/actions/workflows/build-nightly.yml)
[![Build Status (nightly, all features)](https://github.com/sigurd4/bulks/workflows/Build-nightly-all-features/badge.svg)](https://github.com/sigurd4/bulks/actions/workflows/build-nightly-all-features.yml)

[![Build Status (stable)](https://github.com/sigurd4/bulks/workflows/Build-stable/badge.svg)](https://github.com/sigurd4/bulks/actions/workflows/build-stable.yml)
[![Build Status (stable, all features)](https://github.com/sigurd4/bulks/workflows/Build-stable-all-features/badge.svg)](https://github.com/sigurd4/bulks/actions/workflows/build-stable-all-features.yml)

[![Test Status](https://github.com/sigurd4/bulks/workflows/Test/badge.svg)](https://github.com/sigurd4/bulks/actions/workflows/test.yml)
[![Lint Status](https://github.com/sigurd4/bulks/workflows/Lint/badge.svg)](https://github.com/sigurd4/bulks/actions/workflows/lint.yml)

[![Latest Version](https://img.shields.io/crates/v/bulks.svg)](https://crates.io/crates/bulks)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://img.shields.io/docsrs/bulks)](https://docs.rs/bulks)
[![Coverage Status](https://img.shields.io/codecov/c/github/sigurd4/bulks)](https://app.codecov.io/github/sigurd4/bulks)

# bulks

This crate adds `Bulk`s, which are similar to iterators, except they are stricter. They can only be wholly consumed, where every value is operated on in bulk. This, unlike with classic `Iterator`s, makes them fully compatible with arrays!

Their constrained nature means fewer iterator-like operations are possible, but the guarantees it gives makes it possible to use them with arrays while still retaining the array's length. 
Operations that preserves the length of the data like `map`, `zip`, `enumerate`, `rev` and `inspect` are possible. Some other length-modifying operations are also possible with `Bulk`s as long as the length is modified in a predetermined way, like with `flat_map`, `flatten`, `intersperse`, `array_chunks` and `map_windows`.

Any `Bulk` that was created from an array can be collected back into an array, given that the operations done on it makes the length predetermined at compile-time. Bulks can also be used with other structures, allowing generic implementations that will work on arrays as well as other iterables.

# Example

```rust
use bulks::*;

let a = [1, 2, 3];

let b = a.bulk()
    .copied()
    .map(|x| (x - 1) as usize)
    .enumerate()
    .inspect(|(i, x)| assert_eq!(i, x))
    .collect::<[_; _], _>();

assert_eq!(b, [(0, 0), (1, 1), (2, 2)]);
```