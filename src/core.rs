use ::core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::monads::{TokenMap, TokenMapMut};
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

pub trait TokenCellTrait<T: ?Sized, Token: TokenTrait>: Sync {
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
    fn map<'a, U, F: FnOnce(TokenGuard<'a, T, Token>) -> U>(
        &'a self,
        f: F,
    ) -> TokenMap<'a, T, U, F, Self, Token> {
        TokenMap {
            cell: self,
            f,
            marker: core::marker::PhantomData,
        }
    }
    fn map_mut<'a, U, F: FnOnce(TokenGuardMut<'a, T, Token>) -> U>(
        &'a self,
        f: F,
    ) -> TokenMapMut<'a, T, U, F, Self, Token> {
        TokenMapMut {
            cell: self,
            f,
            marker: core::marker::PhantomData,
        }
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
impl<T: ?Sized, Token: TokenTrait> Deref for TokenCell<T, Token> {
    type Target = UnsafeCell<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T: ?Sized, Token: TokenTrait> DerefMut for TokenCell<T, Token> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

unsafe impl<T: ?Sized, Token: TokenTrait> Sync for TokenCell<T, Token> {}

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
