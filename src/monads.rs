use core::convert::Infallible;

use crate::{
    core::{False, TokenGuard, TokenGuardMut, True, UnsafeTokenCellTrait},
    prelude::*,
};

/// An operation waiting to be applied onto a cell by providing a proof of immutable access.
#[must_use = "TokenMaps must be applied to do anything. Note that the closure execution will be deferred to the call-site of `apply/try_apply`"]
pub struct TokenMap<
    'a,
    T: ?Sized,
    U,
    F: FnOnce(TokenGuard<'a, T, Token>) -> U,
    Cell: ?Sized,
    Token: TokenTrait<ComparisonMaySpuriouslyEq = ComparisonMaySpuriouslyEq> + 'a,
    ComparisonMaySpuriouslyEq,
> {
    pub(crate) cell: &'a Cell,
    pub(crate) f: F,
    pub(crate) marker: core::marker::PhantomData<(&'a T, U, Token, ComparisonMaySpuriouslyEq)>,
}
impl<
        'a,
        T: ?Sized,
        U,
        F: FnOnce(TokenGuard<'a, T, Token>) -> U,
        Token: TokenTrait<ComparisonMaySpuriouslyEq = True>,
        Cell: UnsafeTokenCellTrait<T, Token>,
    > TokenMap<'a, T, U, F, Cell, Token, True>
{
    /// Attempt to apply the operation.
    ///
    /// # Errors
    /// If the token comparison failed. Reaching this error is likely to be a fundamental error in your program.
    pub fn try_apply(self, token: &'a Token) -> Result<U, (Self, Token::ComparisonError)> {
        match unsafe { self.cell.try_guard(token) } {
            Ok(borrowed) => Ok((self.f)(borrowed)),
            Err(e) => Err((self, e)),
        }
    }
    /// Applies the operation.
    pub fn apply(self, token: &'a Token) -> U
    where
        Token: TokenTrait<ComparisonError = Infallible>,
    {
        let borrowed = unsafe { self.cell.try_guard(token).unwrap_unchecked() };
        (self.f)(borrowed)
    }
}

impl<
        'a,
        T: ?Sized,
        U,
        F: FnOnce(TokenGuard<'a, T, Token>) -> U,
        Token: TokenTrait<ComparisonMaySpuriouslyEq = False>,
        Cell: TokenCellTrait<T, Token>,
    > TokenMap<'a, T, U, F, Cell, Token, False>
{
    /// Attempt to apply the operation.
    ///
    /// # Errors
    /// If the token comparison failed. Reaching this error is likely to be a fundamental error in your program.
    pub fn try_apply(self, token: &'a Token) -> Result<U, (Self, Token::ComparisonError)> {
        match self.cell.try_guard(token) {
            Ok(borrowed) => Ok((self.f)(borrowed)),
            Err(e) => Err((self, e)),
        }
    }
    /// Applies the operation.
    pub fn apply(self, token: &'a Token) -> U
    where
        Token: TokenTrait<ComparisonError = Infallible>,
    {
        let Ok(borrowed) = self.cell.try_guard(token);
        (self.f)(borrowed)
    }
}

/// An operation waiting to be applied onto a cell by providing a proof of mutable access.
#[must_use = "TokenMaps must be applied to do anything. Note that the closure execution will be deferred to the call-site of `apply/try_apply`"]
pub struct TokenMapMut<
    'a,
    T: ?Sized,
    U,
    F: FnOnce(TokenGuardMut<'a, T, Token>) -> U,
    Cell: ?Sized,
    Token: TokenTrait<ComparisonMaySpuriouslyEq = ComparisonMaySpuriouslyEq> + 'a,
    ComparisonMaySpuriouslyEq,
> {
    pub(crate) cell: &'a Cell,
    pub(crate) f: F,
    pub(crate) marker: core::marker::PhantomData<(&'a T, U, Token, ComparisonMaySpuriouslyEq)>,
}
impl<
        'a,
        T: ?Sized,
        U,
        F: FnOnce(TokenGuardMut<'a, T, Token>) -> U,
        Token: TokenTrait<ComparisonMaySpuriouslyEq = False>,
        Cell: TokenCellTrait<T, Token>,
    > TokenMapMut<'a, T, U, F, Cell, Token, False>
{
    /// Attempt to apply the operation.
    ///
    /// # Errors
    /// If the token comparison failed. Reaching this error is likely to be a fundamental error in your program.
    pub fn try_apply(self, token: &'a mut Token) -> Result<U, Token::ComparisonError> {
        let borrowed = self.cell.try_guard_mut(token)?;
        Ok((self.f)(borrowed))
    }
    /// Applies the operation.
    pub fn apply(self, token: &'a mut Token) -> U
    where
        Token: TokenTrait<ComparisonError = Infallible>,
    {
        let Ok(borrowed) = self.cell.try_guard_mut(token);
        (self.f)(borrowed)
    }
}
impl<
        'a,
        T: ?Sized,
        U,
        F: FnOnce(TokenGuardMut<'a, T, Token>) -> U,
        Token: TokenTrait<ComparisonMaySpuriouslyEq = True>,
        Cell: UnsafeTokenCellTrait<T, Token>,
    > TokenMapMut<'a, T, U, F, Cell, Token, True>
{
    /// Attempt to apply the operation.
    ///
    /// # Errors
    /// If the token comparison failed. Reaching this error is likely to be a fundamental error in your program.
    pub fn try_apply(self, token: &'a mut Token) -> Result<U, Token::ComparisonError> {
        let borrowed = unsafe { self.cell.try_guard_mut(token) }?;
        Ok((self.f)(borrowed))
    }
    /// Applies the operation.
    pub fn apply(self, token: &'a mut Token) -> U
    where
        Token: TokenTrait<ComparisonError = Infallible>,
    {
        let Ok(borrowed) = unsafe { self.cell.try_guard_mut(token) };
        (self.f)(borrowed)
    }
}
