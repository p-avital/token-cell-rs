use ::core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

trait MapLikely<T> {
    type Output<U>;
    fn map_likely<U, F: FnOnce(T) -> U>(self, f: F) -> Self::Output<U>;
}
#[cold]
const fn cold() {}
impl<T, E> MapLikely<T> for Result<T, E> {
    type Output<U> = Result<U, E>;
    fn map_likely<U, F: FnOnce(T) -> U>(self, f: F) -> Self::Output<U> {
        match self {
            Ok(v) => Ok(f(v)),
            Err(e) => {
                cold();
                Err(e)
            }
        }
    }
}

use crate::monads::{TokenMap, TokenMapMut};
/// A trait for tokens
pub trait TokenTrait: Sized {
    /// Constructing a token may fail.
    type ConstructionError;
    /// [`TokenTrait::with_token`] may fail, typically if construction failed.
    ///
    /// Some types, like [`GhostToken`](crate::ghost::GhostToken) are inconstructible with [`TokenTrait::new`], but cannot fail to run, hence the distinction.
    type RunError;
    /// Lets a [`TokenCell`] keep track of the token.
    ///
    /// In most cases, this is a ZST in release mode.
    type Identifier;
    /// [`core::convert::Infallible`] unless [`TokenTrait::compare`] is fallible (ie. comparison is done at runtime).
    type ComparisonError;
    /// Rebrands the token, this is necessary for [`GhostToken`](crate::ghost::GhostToken) to function properly
    type Branded<'a>;
    /// Constructs a new Token.
    ///
    /// # Errors
    /// Some token implementations may chose to be constructible only once, or only while holding a given lock.
    fn new() -> Result<Self, Self::ConstructionError>;
    /// Constructs a new, lifetime-branded Token, and provides it to the closure.
    ///
    /// This is especially useful for [`GhostToken`](crate::ghost::GhostToken), which can only be constructed that way, as they use lifetimes to obtain a unique brand.
    ///
    /// # Errors
    /// If construction failed, so will this.
    fn with_token<R, F: for<'a> FnOnce(Self::Branded<'a>) -> R>(f: F) -> Result<R, Self::RunError>;
    /// Returns the Token's identifier, which cells may store to allow comparison.
    fn identifier(&self) -> Self::Identifier;
    /// Allows the cell to compare its identifier to the Token.
    ///
    /// # Errors
    /// If a wrong token was mistakenly passed to the cell.
    fn compare(&self, id: &Self::Identifier) -> Result<(), Self::ComparisonError>;
}

/// Common ways to interract with a [`TokenCell`].
///
/// Note that while many functions document fallihle behaviours, this behaviour is only reachable for tokens that perform runtime check. These are identifiable by their [`TokenTrait::ComparisonError`] type not being [`core::convert::Infallible`].
pub trait TokenCellTrait<T: ?Sized, Token: TokenTrait>: Sync {
    /// Constructs a new cell using `token` as its key.
    fn new(inner: T, token: &Token) -> Self
    where
        T: Sized;
    /// Attempts to construct a guard which [`Deref`]s to the inner data,
    /// but also allows recovering the `Token`.
    ///
    /// # Errors
    /// If the token provides runtime checking and detects that `self` was constructed with another token.
    fn try_guard<'l>(
        &'l self,
        token: &'l Token,
    ) -> Result<TokenGuard<'l, T, Token>, Token::ComparisonError>;
    /// Attempts to borrow the inner data.
    ///
    /// # Errors
    /// If the token provides runtime checking and detects that `self` was constructed with another token.
    fn try_borrow<'l>(&'l self, token: &'l Token) -> Result<&'l T, Token::ComparisonError>;
    /// Attempts to construct a guard which [`DerefMut`]s to the inner data,
    /// but also allows recovering the `Token`.
    ///
    /// # Errors
    /// If the token provides runtime checking and detects that `self` was constructed with another token.
    fn try_guard_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<TokenGuardMut<'l, T, Token>, Token::ComparisonError>;
    /// Attempts to borrow the inner data mutably.
    ///
    /// # Errors
    /// If the token provides runtime checking and detects that `self` was constructed with another token.
    fn try_borrow_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<&'l mut T, Token::ComparisonError>;
    /// Borrows the inner data, panicking if the wrong token was used as key.
    fn borrow<'l>(&'l self, token: &'l Token) -> &'l T
    where
        Token::ComparisonError: core::fmt::Debug,
    {
        self.try_borrow(token).unwrap()
    }
    /// Borrows the inner data mutably, panicking if the wrong token was used as key.
    fn borrow_mut<'l>(&'l self, token: &'l mut Token) -> &'l mut T
    where
        Token::ComparisonError: core::fmt::Debug,
    {
        self.try_borrow_mut(token).unwrap()
    }
    /// Constructs a lazy computation that can then be applied using the token.
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
    /// Constructs a lazy computation that can then be applied using the token.
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

/// A guard that allows immutably borrowing the cell's value, as well as its token.
pub struct TokenGuard<'a, T: ?Sized, Token: TokenTrait> {
    cell: &'a TokenCell<T, Token>,
    token: &'a Token,
}
impl<'a, T: ?Sized, Token: TokenTrait> TokenGuard<'a, T, Token> {
    /// Reborrows the token immutably.
    pub const fn token(&self) -> &Token {
        self.token
    }
}
impl<'a, T: ?Sized, Token: TokenTrait> Deref for TokenGuard<'a, T, Token> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.inner.get() }
    }
}

/// A guard that allows mutably borrowing the cell's value, as well as its token.
pub struct TokenGuardMut<'a, T: ?Sized, Token: TokenTrait> {
    cell: &'a TokenCell<T, Token>,
    token: &'a mut Token,
}
impl<'a, T: ?Sized, Token: TokenTrait> TokenGuardMut<'a, T, Token> {
    /// Reborrows the token immutably.
    pub const fn token(&self) -> &Token {
        self.token
    }
    /// Reborrows the token mutably.
    pub const fn token_mut(&mut self) -> &mut Token {
        self.token
    }
}
impl<'a, T: ?Sized, Token: TokenTrait> Deref for TokenGuardMut<'a, T, Token> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.inner.get() }
    }
}
impl<'a, T, Token: TokenTrait> core::ops::DerefMut for TokenGuardMut<'a, T, Token> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.cell.inner.get() }
    }
}

/// A Cell that shifts the management of access permissions to its inner value onto a `Token`.
pub struct TokenCell<T: ?Sized, Token: TokenTrait> {
    token_id: Token::Identifier,
    inner: UnsafeCell<T>,
}
impl<T: ?Sized, Token: TokenTrait> TokenCell<T, Token> {
    /// While cells are typically behind immutable references,
    /// obtaining a mutable reference to one is still proof of unique access.
    pub const fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}
impl<T: Sized, Token: TokenTrait> TokenCell<T, Token> {
    /// Unwraps the value from the cell.
    ///
    /// Full ownership of the cell is sufficient proof that the inner value can be recovered.
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
    fn try_guard<'l>(
        &'l self,
        token: &'l Token,
    ) -> Result<TokenGuard<'l, T, Token>, <Token as TokenTrait>::ComparisonError> {
        token
            .compare(&self.token_id)
            .map_likely(move |_| TokenGuard { cell: self, token })
    }
    fn try_borrow<'l>(&'l self, token: &'l Token) -> Result<&'l T, Token::ComparisonError> {
        token
            .compare(&self.token_id)
            .map_likely(move |_| unsafe { &*self.inner.get() })
    }
    fn try_guard_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<TokenGuardMut<'l, T, Token>, <Token as TokenTrait>::ComparisonError> {
        token
            .compare(&self.token_id)
            .map_likely(move |_| TokenGuardMut { cell: self, token })
    }

    fn try_borrow_mut<'l>(
        &'l self,
        token: &'l mut Token,
    ) -> Result<&'l mut T, Token::ComparisonError> {
        token
            .compare(&self.token_id)
            .map_likely(move |_| unsafe { &mut *self.inner.get() })
    }
}
