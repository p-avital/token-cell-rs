//! This library provides an alternative to [`ghost-cell`](https://crates.io/crates/ghost-cell) which uses concrete types instead of lifetimes for branding.
//!
//! This allows a more convenient usage, where cells and tokens can be constructed independently, with the same compile-time guarantees as [`ghost-cell`](https://crates.io/crates/ghost-cell). The trade-off for this arguably more convenient usage and arguably easier to understand branding method is that tokens, while zero-sized if made correctly, must be guaranteed to be constructable only if no other instance exists.
//!
//! To this end, this crate provides the [`generate_token`] macro, which will create a ZST which can only be constructed using [`TokenTrait::aquire`], which is generated to guarantee no other token exists before returning the token. This is done by checking a static `AtomicBool` flag, which is the only runtime cost of these tokens.
#![no_std]
use core::{cell::UnsafeCell, convert::Infallible};
#[cfg(not(features = "no_std"))]
mod std {
    use crate::IdMismatch;
    extern crate std;
    impl std::error::Error for IdMismatch {}
}
pub mod support;
pub use support::{IdMismatch, RuntimeToken, TokenChecker, TokenTrait};

/// A cell which requires a token for interior read/write access.
#[repr(C)]
pub struct TokenCell<Token: TokenTrait, T> {
    inner: UnsafeCell<T>,
    checker: Token::Checker,
}
impl<Token: TokenTrait, T> TokenCell<Token, T>
where
    Token::Checker: TokenChecker<Token, Error = Infallible>,
{
    #[inline(always)]
    /// Borrows the cell's content.
    pub fn borrow<'l>(&'l self, _: &'l Token) -> &'l T {
        unsafe { &*self.inner.get() }
    }
    #[inline(always)]
    /// Borrows the cell's content mutably.
    pub fn borrow_mut<'l>(&'l self, _: &'l mut Token) -> &'l mut T {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    /// Places `value` inside the cell, returning the
    pub fn swap(&self, mut value: T, token: &mut Token) -> T {
        core::mem::swap(self.borrow_mut(token), &mut value);
        value
    }
}
impl<Token: TokenTrait, T> TokenCell<Token, T> {
    /// Builds a [`TokenCell`] attached to a `token`
    #[inline(always)]
    pub fn new(value: T) -> Self
    where
        Token::Checker: TokenChecker<Token, Input = ()>,
    {
        TokenCell {
            inner: UnsafeCell::new(value),
            checker: TokenChecker::new(()),
        }
    }
    /// Builds a [`TokenCell`] attached to a `token`
    #[inline(always)]
    pub fn with_token(token: &Token, value: T) -> Self
    where
        Token::Checker: TokenChecker<Token>,
    {
        TokenCell {
            inner: UnsafeCell::new(value),
            checker: TokenChecker::from_ref(token),
        }
    }
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
    #[inline(always)]
    /// Fallible equivalent of [`TokenCell::swap`], for runtime-checked tokens.
    ///
    /// Since token mismatch is indicative that the wrong token was used to interact with the cell,
    /// you should treat these as unrecoverable errors by unwrapping the result.
    pub fn try_swap(
        &self,
        mut value: T,
        token: &mut Token,
    ) -> Result<T, <Token::Checker as TokenChecker<Token>>::Error> {
        core::mem::swap(self.try_borrow_mut(token)?, &mut value);
        Ok(value)
    }
    #[inline(always)]
    /// Fallible equivalent of [`TokenCell::borrow`], for runtime-checked tokens.
    ///
    /// Since token mismatch is indicative that the wrong token was used to interact with the cell,
    /// you should treat these as unrecoverable errors by unwrapping the result.
    pub fn try_borrow<'l>(
        &'l self,
        token: &'l Token,
    ) -> Result<&'l T, <Token::Checker as TokenChecker<Token>>::Error> {
        self.checker
            .check(token)
            .map(|_| unsafe { &*self.inner.get() })
    }
    #[inline(always)]
    /// Fallible equivalent of [`TokenCell::borrow_mut`], for runtime-checked tokens.
    ///
    /// Since token mismatch is indicative that the wrong token was used to interact with the cell,
    /// you should treat these as unrecoverable errors by unwrapping the result.
    pub fn try_borrow_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<&'l mut T, <Token::Checker as TokenChecker<Token>>::Error> {
        self.checker
            .check(token)
            .map(|_| unsafe { &mut *self.inner.get() })
    }
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut T {
        self.inner.get()
    }
}

impl<Token: TokenTrait, T> AsMut<T> for TokenCell<Token, T> {
    fn as_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}
unsafe impl<Token: TokenTrait, T: Send> Send for TokenCell<Token, T> {}
unsafe impl<Token: TokenTrait, T: Sync> Sync for TokenCell<Token, T> {}
