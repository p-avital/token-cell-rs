//! This library provides an alternative to [`ghost-cell`](https://crates.io/crates/ghost-cell) which uses concrete types instead of lifetimes for branding.
//!
//! This allows a more convenient usage, where cells and tokens can be constructed independently, with the same compile-time guarantees as [`ghost-cell`](https://crates.io/crates/ghost-cell). The trade-off for this arguably more convenient usage and arguably easier to understand branding method is that tokens, while zero-sized if made correctly, must be guaranteed to be constructable only if no other instance exists.
//!
//! To this end, this crate provides the [`generate_token`] macro, which will create a ZST which can only be constructed using [`TokenTrait::aquire`], which is generated to guarantee no other token exists before returning the token. This is done by checking a static `AtomicBool` flag, which is the only runtime cost of these tokens.
#![no_std]
use core::{cell::UnsafeCell, convert::Infallible, ops::Deref};
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
    fn with_token<R, F: FnOnce(Self) -> R>(f: F) -> Result<R, Self::RunError>;
    fn identifier(&self) -> Self::Identifier;
    fn compare(&self, id: &Self::Identifier) -> Result<(), Self::ComparisonError>;
}

pub trait TokenCellTrait<T: ?Sized, Token: TokenTrait> {
    fn new(inner: T, token: &Token) -> Self
    where
        T: Sized;
    fn try_borrow<'l>(
        &'l self,
        token: &'l Token,
    ) -> Result<TokenGuard<'l, T, Token>, Token::ComparisonError>;
    fn try_borrow_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<TokenGuardMut<'l, T, Token>, Token::ComparisonError>;
    fn borrow<'l>(&'l self, token: &'l Token) -> TokenGuard<'l, T, Token>
    where
        Token::ComparisonError: core::fmt::Debug,
    {
        self.try_borrow(token).unwrap()
    }
    fn borrow_mut<'l>(&'l self, token: &'l mut Token) -> TokenGuardMut<'l, T, Token>
    where
        Token::ComparisonError: core::fmt::Debug,
    {
        self.try_borrow_mut(token).unwrap()
    }
}

pub struct TokenGuard<'a, T: ?Sized, Token: TokenTrait> {
    cell: &'a TokenCell<T, Token>,
    token: &'a Token,
}
impl<'a, T: ?Sized, Token: TokenTrait> TokenGuard<'a, T, Token> {
    pub fn token(&'a self) -> &'a Token {
        self.token
    }
}
impl<'a, T: ?Sized, Token: TokenTrait> Deref for TokenGuard<'a, T, Token> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.inner.get().cast_const() }
    }
}

pub struct TokenGuardMut<'a, T: ?Sized, Token: TokenTrait> {
    cell: &'a TokenCell<T, Token>,
    token: &'a mut Token,
}
impl<'a, T: ?Sized, Token: TokenTrait> TokenGuardMut<'a, T, Token> {
    pub fn token(&'a self) -> &'a Token {
        self.token
    }
    pub fn token_mut(&'a mut self) -> &'a mut Token {
        self.token
    }
}
impl<'a, T: ?Sized, Token: TokenTrait> Deref for TokenGuardMut<'a, T, Token> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.inner.get().cast_const() }
    }
}
impl<'a, T, Token: TokenTrait> core::ops::DerefMut for TokenGuardMut<'a, T, Token> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.cell.inner.get() }
    }
}

pub struct TokenCell<T: ?Sized, Token: TokenTrait> {
    token_id: Token::Identifier,
    inner: UnsafeCell<T>,
}
impl<T: ?Sized, Token: TokenTrait> TokenCell<T, Token> {
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}
impl<T: Sized, Token: TokenTrait> TokenCell<T, Token> {
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T: ?Sized, Token: TokenTrait> TokenCellTrait<T, Token> for TokenCell<T, Token> {
    fn new(inner: T, token: &Token) -> Self
    where
        T: Sized,
    {
        TokenCell {
            inner: UnsafeCell::new(inner),
            token_id: token.identifier(),
        }
    }

    fn try_borrow<'l>(
        &'l self,
        token: &'l Token,
    ) -> Result<TokenGuard<'l, T, Token>, Token::ComparisonError> {
        token
            .compare(&self.token_id)
            .map(move |_| TokenGuard { cell: self, token })
    }

    fn try_borrow_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<TokenGuardMut<'l, T, Token>, Token::ComparisonError> {
        token
            .compare(&self.token_id)
            .map(move |_| TokenGuardMut { cell: self, token })
    }
}

#[derive(Clone, Copy)]
pub struct InvariantLifetime<'a>(core::marker::PhantomData<UnsafeCell<&'a ()>>);
impl<'a> InvariantLifetime<'a> {
    const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

/// WARNING: This attempt at recreating GhostCell but with traits does NOT work.
///
/// I am leaving this here because I believe there may exist a way to make this type of
/// token work, and anyone who has ideas of how to do so is welcome to try and make a PR.
///
/// To check your theory, clone this repo and use the `ghost.rs` example as a check for
/// your attempt. If your method works, the example should have some compile error.
pub struct GhostToken<'brand>(InvariantLifetime<'brand>);
impl<'brand> TokenTrait for GhostToken<'brand> {
    type ConstructionError = ();
    type RunError = Infallible;
    type Identifier = InvariantLifetime<'brand>;
    type ComparisonError = Infallible;
    fn new() -> Result<Self, Self::ConstructionError> {
        Err(())
    }
    fn with_token<R, F: FnOnce(Self) -> R>(f: F) -> Result<R, Self::RunError> {
        Ok(f(Self(InvariantLifetime::new())))
    }

    fn identifier(&self) -> InvariantLifetime<'brand> {
        self.0
    }

    fn compare(&self, _: &InvariantLifetime<'brand>) -> Result<(), Self::ComparisonError> {
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
                    type ComparisonError = $crate::IdMismatch;
                    fn new() -> Result<Self, Self::ConstructionError> {
                        Ok($id(
                            COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed),
                        ))
                    }
                    fn with_token<R, F: FnOnce(Self)->R>(f: F) -> Result<R, Self::RunError> {
                        Self::new().map(f)
                    }
                    fn identifier(&self) -> Self::Identifier {
                        self.0
                    }
                    fn compare(&self, id: &Self::Identifier) -> Result<(), Self::ComparisonError> {
                        if self.0 == *id {
                            Ok(())
                        } else {
                            Err($crate::IdMismatch {
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
                    fn with_token<R, F: FnOnce(Self)->R>(f: F) -> Result<R, Self::RunError> {
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
                    fn with_token<R, F: FnOnce(Self)->R>(f: F) -> Result<R, Self::RunError> {
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
