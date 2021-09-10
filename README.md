This library provides an alternative to [`ghost-cell`](https://crates.io/crates/ghost-cell) which uses concrete types instead of lifetimes for branding.

This allows a more convenient usage, where cells and tokens can be constructed independently, with the same compile-time guarantees as [`ghost-cell`](https://crates.io/crates/ghost-cell). The trade-off for this arguably more convenient usage and arguably easier to understand branding method is that tokens, while zero-sized if made correctly, must be guaranteed to be constructable only if no other instance exists.

To this end, this crate provides the `generate_token` macro, which will create a ZST which can only be constructed using `TokenTrait::new`.