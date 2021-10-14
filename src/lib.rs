//! This library provides an alternative to [`ghost-cell`](https://crates.io/crates/ghost-cell) which uses concrete types instead of lifetimes for branding.
//!
//! This allows a more convenient usage, where cells and tokens can be constructed independently, with the same compile-time guarantees as [`ghost-cell`](https://crates.io/crates/ghost-cell). The trade-off for this arguably more convenient usage and arguably easier to understand branding method is that tokens, while zero-sized if made correctly, must be guaranteed to be constructable only if no other instance exists.
//!
//! To this end, this crate provides the [`generate_token`] macro, which will create a ZST which can only be constructed using [`TokenTrait::aquire`], which is generated to guarantee no other token exists before returning the token. This is done by checking a static `AtomicBool` flag, which is the only runtime cost of these tokens.
#![no_std]
use core::{cell::UnsafeCell, convert::Infallible};
pub use paste::paste;
#[cfg(not(features = "no_std"))]
mod std {
    use crate::{IdMismatch, SingletonUnavailable};
    extern crate std;
    impl std::error::Error for IdMismatch {}
    impl std::error::Error for SingletonUnavailable {}
}

/// A trait for tokens
pub trait TokenTrait: Sized {
    type ConstructionError;
    type RunError;
    type Identifier;
    type ComparisonError;
    fn new() -> Result<Self, Self::ConstructionError>;
    fn with_token<F: FnOnce(Self)>(f: F) -> Result<(), Self::RunError>;
    fn identifier(&self) -> Self::Identifier;
    fn compare(&self, id: &Self::Identifier) -> Result<(), Self::ComparisonError>;
}

pub trait TokenCellTrait<T, Token: TokenTrait> {
    fn new(inner: T, token: &Token) -> Self;
    fn try_borrow<'l>(&'l self, token: &'l Token) -> Result<&'l T, Token::ComparisonError>;
    fn try_borrow_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<&'l mut T, Token::ComparisonError>;
    fn borrow<'l>(&'l self, token: &'l Token) -> &'l T
    where
        Token::ComparisonError: core::fmt::Debug,
    {
        self.try_borrow(token).unwrap()
    }
    fn borrow_mut<'l>(&'l self, token: &'l mut Token) -> &'l mut T
    where
        Token::ComparisonError: core::fmt::Debug,
    {
        self.try_borrow_mut(token).unwrap()
    }
}

pub struct TokenCell<T, Token: TokenTrait> {
    inner: UnsafeCell<T>,
    token_id: Token::Identifier,
}
impl<T, Token: TokenTrait> TokenCell<T, Token> {
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T, Token: TokenTrait> TokenCellTrait<T, Token> for TokenCell<T, Token> {
    fn new(inner: T, token: &Token) -> Self {
        TokenCell {
            inner: UnsafeCell::new(inner),
            token_id: token.identifier(),
        }
    }

    fn try_borrow<'l>(&'l self, token: &'l Token) -> Result<&'l T, Token::ComparisonError> {
        token
            .compare(&self.token_id)
            .map(|_| unsafe { &*self.inner.get() })
    }

    fn try_borrow_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<&'l mut T, Token::ComparisonError> {
        token
            .compare(&self.token_id)
            .map(|_| unsafe { &mut *self.inner.get() })
    }
}

pub struct GhostToken<'brand>(core::marker::PhantomData<&'brand ()>);
impl<'brand> TokenTrait for GhostToken<'brand> {
    type ConstructionError = ();
    type RunError = Infallible;
    type Identifier = ();
    type ComparisonError = Infallible;
    fn new() -> Result<Self, Self::ConstructionError> {
        Err(())
    }
    fn with_token<F: FnOnce(Self)>(f: F) -> Result<(), Self::RunError> {
        f(GhostToken(Default::default()));
        Ok(())
    }

    fn identifier(&self) -> Self::Identifier {}

    fn compare(&self, _: &Self::Identifier) -> Result<(), Self::ComparisonError> {
        Ok(())
    }
}

#[macro_export]
macro_rules! runtime_token {
    ($vis: vis $id: ident) => {
        $crate::paste! {
            $vis use [<__ $id _mod__ >]::$id;
            #[allow(nonstandard_style)]
            mod [<__ $id _mod__ >] {
                use core::{convert::Infallible, sync::atomic::AtomicUsize};
                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                pub struct $id(usize);
                impl $crate::TokenTrait for $id {
                    type ConstructionError = Infallible;
                    type RunError = Infallible;
                    type Identifier = usize;
                    type ComparisonError = crate::IdMismatch;
                    fn new() -> Result<Self, Self::ConstructionError> {
                        Ok($id(
                            COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed),
                        ))
                    }
                    fn with_token<F: FnOnce(Self)>(f: F) -> Result<(), Self::RunError> {
                        Self::new().map(f)
                    }
                    fn identifier(&self) -> Self::Identifier {
                        self.0
                    }
                    fn compare(&self, id: &Self::Identifier) -> Result<(), Self::ComparisonError> {
                        if self.0 == *id {
                            Ok(())
                        } else {
                            Err(crate::IdMismatch {
                                cell: *id,
                                token: self.0,
                            })
                        }
                    }
                }
            }
        }
    };
    ($($vis: vis $id: ident),*) => {
        $(runtime_token!($vis $id);)*
    }
}

#[macro_export]
macro_rules! singleton_token {
    ($vis: vis $id: ident) => {
        $crate::paste! {
            $vis use [<__ $id _mod__ >]::$id;
            #[allow(nonstandard_style)]
            mod [<__ $id _mod__ >] {
                use core::{convert::Infallible, sync::atomic::AtomicBool};
                use $crate::SingletonUnavailable;
                static AVAILABLE: AtomicBool = AtomicBool::new(true);
                pub struct $id(());
                impl $crate::TokenTrait for $id {
                    type ConstructionError = SingletonUnavailable;
                    type RunError = SingletonUnavailable;
                    type Identifier = ();
                    type ComparisonError = Infallible;
                    fn new() -> Result<Self, Self::ConstructionError> {
                        if AVAILABLE.swap(false, core::sync::atomic::Ordering::Relaxed) {
                            Ok($id(()))
                        } else {
                            Err(SingletonUnavailable)
                        }
                    }
                    fn with_token<F: FnOnce(Self)>(f: F) -> Result<(), Self::RunError> {
                        Self::new().map(f)
                    }
                    fn identifier(&self) -> Self::Identifier {
                        self.0
                    }
                    fn compare(&self, _: &Self::Identifier) -> Result<(), Self::ComparisonError> {
                        Ok(())
                    }
                }
            }
        }
    };
    ($($vis: vis $id: ident),*) => {
        $(singleton_token!($vis $id);)*
    }
}

#[macro_export]
macro_rules! unsafe_token {
    ($vis: vis $id: ident) => {
        $crate::paste! {
            $vis use [<__ $id _mod__ >]::$id;
            #[allow(nonstandard_style)]
            mod [<__ $id _mod__ >] {
                use core::convert::Infallible;
                pub struct $id(());
                impl $crate::TokenTrait for $id {
                    type ConstructionError = Infallible;
                    type RunError = Infallible;
                    type Identifier = ();
                    type ComparisonError = Infallible;
                    fn new() -> Result<Self, Self::ConstructionError> {
                        Ok($id(()))
                    }
                    fn with_token<F: FnOnce(Self)>(f: F) -> Result<(), Self::RunError> {
                        Self::new().map(f)
                    }
                    fn identifier(&self) -> Self::Identifier {
                        self.0
                    }
                    fn compare(&self, _: &Self::Identifier) -> Result<(), Self::ComparisonError> {
                        Ok(())
                    }
                }
            }
        }
    };
    ($($vis: vis $id: ident),*) => {
        $(unsafe_token!($vis $id);)*
    }
}
pub use token::token;
#[cfg(feature = "debug")]
mod token {
    pub use super::runtime_token as token;
}
#[cfg(not(feature = "debug"))]
mod token {
    pub use super::unsafe_token as token;
}

#[derive(Debug, Clone, Copy)]
pub struct IdMismatch {
    pub cell: usize,
    pub token: usize,
}
impl core::fmt::Display for IdMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct SingletonUnavailable;
impl core::fmt::Display for SingletonUnavailable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
runtime_token!(pub RuntimeToken);
