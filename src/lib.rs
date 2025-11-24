#![deny(
    clippy::missing_panics_doc,
    clippy::missing_const_for_fn,
    clippy::missing_safety_doc,
    clippy::missing_errors_doc,
    missing_docs
)]

//! This library provides an alternative to [`ghost-cell`](https://crates.io/crates/ghost-cell) which uses concrete types instead of lifetimes for branding.
//!
//! This allows a more convenient usage, where cells and tokens can be constructed independently, with the same compile-time guarantees as [`ghost-cell`](https://crates.io/crates/ghost-cell). The trade-off for this arguably more convenient usage and arguably easier to understand branding method is that tokens, while zero-sized if made correctly, must be guaranteed to be constructable only if no other instance exists.
#![cfg_attr(not(feature = "std"), no_std)]
pub use paste::paste;
#[cfg(feature = "std")]
mod std {
    use crate::macros::{IdMismatch, SingletonUnavailable};
    extern crate std;
    impl std::error::Error for IdMismatch {}
    impl std::error::Error for SingletonUnavailable {}
}
/// The basis for using `token_cell`
pub mod prelude {
    pub use crate::core::{TokenCell, TokenCellTrait, TokenTrait};
}
pub use crate::macros::token;
/// The core aspects of `token_cell`
pub mod core;
/// A traitified version of `ghost_cell`.
///
/// To use this, simply construct a [`TokenCell`](crate::prelude::TokenCell) using a [`GhostToken`](crate::ghost::GhostToken) obtained with the [`TokenTrait::with_token`](crate::prelude::TokenTrait::with_token) constructor.
pub mod ghost;
/// The macros to construct tokens.
pub mod macros;
/// Because monads are cool.
pub mod monads;
/// Re-export the portable-atomic crate because it is used in the macros.
pub mod atomics {
    pub use portable_atomic::AtomicU16;
}

runtime_token!(pub RuntimeToken);
