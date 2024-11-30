<h1 align="center">
  SingletonSet
</h1>

<p align="center">
  A Data Structure for Unique Type Instances
</p>

<p align="center">
<a
  href="https://github.com/FlippingBinaryLLC/singletonset-rs/actions?query=branch%3Amain"><img
    alt="Build Status"
    src="https://img.shields.io/github/actions/workflow/status/FlippingBinaryLLC/singletonset-rs/ci.yml?branch=main"></a>
<a
  href="https://crates.io/crates/singletonset"><img alt="Latest Release on crates.io"
  src="https://img.shields.io/crates/v/singletonset.svg"></a>
</p>

<p align="center">
<a href="https://docs.rs/singletonset">
  Documentation
</a>
  -
<a href="https://github.com/FlippingBinaryLLC/singletonset-rs">
  Website
</a>
</p>

This crate provides the `SingletonSet` data structure, which makes it easy to
store a single instance each of various types within a single set.

This data structure can be used to create a locally-scoped Singleton out of
any data types within it. It ensures there is only one instance of any type,
similar to a Singleton, without polluting the global scope.

## Project Status

This is an alpha release. This project is under active development for a very
specific use-case. Even though the API is not yet stable, this package is
being shared in case others find it useful or want to offer suggestions for
how to overcome certain challenges.

In this early release stage, there are inevitably going to be bugs. Please
don't use this for anything critical yet.

Also, the name of the crate and the data structure may change at some point.
`SingletonSet` seemed fitting, but it is a bit long. Suggestions are welcome!

## Features

- **Type Safety:** Ensures that only one value per type is present in the set
  at all times, using Rust's type system.
- **Flexible Initialization:** Types that implement `Default` can be
  automatically initialized, or any type can be initialized from a value or
  closure for fully customizable initialization.
- **Global Safety:** In contrast to global singletons, a `SingletonSet`
  object can be scoped as needed.

## Example Usage

```rust
use singletonset::SingletonSet;

fn main() {
    let mut set = SingletonSet::new();

    // Initialize from a default value
    set.insert_default::<u32>();

    // Initialize from a closure
    set.insert_with(|| "Hello".to_string());

    // Access and modify values
    set.with_mut(|val: &mut u32| *val += 2);
    set.with_mut::<u32, _>(|val| *val *= 3);
    set.with_mut(|val: &mut String| *val += ", World!");

    // The type must never be ambiguous, but can be inferred.
    *set.as_mut() = 35.77f64;

    println!("u32: {}, f64: {}, String: {}",
      set.as_ref() as &u32,
      set.as_ref() as &f64,
      set.as_ref() as &String
    );
}
```

## Installation

Use `cargo add singletonset` or add the following dependency to your
`Cargo.toml` file:

```toml
[dependencies]
singletonset = "0.1"
```

## Contributing

Contributions are welcome! Please [open an issue] or submit a pull request if
you have any suggestions, bug reports, or feature requests.

## License

Licensed under either of the [Apache License, Version 2.0][APACHE-2.0] or the
[MIT license][MIT] at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[open an issue]: https://github.com/FlippingBinaryLLC/wait-rs/issues
[APACHE-2.0]: https://www.apache.org/licenses/LICENSE-2.0
[MIT]: https://opensource.org/licenses/MIT
