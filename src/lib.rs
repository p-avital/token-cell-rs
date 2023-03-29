//! This library provides an alternative to [`ghost-cell`](https://crates.io/crates/ghost-cell) which uses concrete types instead of lifetimes for branding.
//!
//! This allows a more convenient usage, where cells and tokens can be constructed independently, with the same compile-time guarantees as [`ghost-cell`](https://crates.io/crates/ghost-cell). The trade-off for this arguably more convenient usage and arguably easier to understand branding method is that tokens, while zero-sized if made correctly, must be guaranteed to be constructable only if no other instance exists.
//!
//! To this end, this crate provides the [`generate_token`] macro, which will create a ZST which can only be constructed using [`TokenTrait::aquire`], which is generated to guarantee no other token exists before returning the token. This is done by checking a static `AtomicBool` flag, which is the only runtime cost of these tokens.
#![cfg_attr(not(features = "std"), no_std)]
pub use paste::paste;
#[cfg(features = "std")]
mod std {
    use crate::macros::{IdMismatch, SingletonUnavailable};
    extern crate std;
    impl std::error::Error for IdMismatch {}
    impl std::error::Error for SingletonUnavailable {}
}
pub mod prelude {
    pub use crate::core::{TokenCell, TokenCellTrait, TokenTrait};
}
pub use crate::macros::token;
pub mod core;
pub mod ghost;
pub mod macros;
pub mod monads;

runtime_token!(pub RuntimeToken);
