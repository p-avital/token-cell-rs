use core::convert::Infallible;

use crate::{
    core::{TokenGuard, TokenGuardMut},
    prelude::*,
};
#[must_use = "TokenMaps must be applied to do anything. Note that the closure execution will be deferred to the call-site of `apply/try_apply`"]
pub struct TokenMap<
    'a,
    T: ?Sized,
    U,
    F: FnOnce(TokenGuard<'a, T, Token>) -> U,
    Cell: TokenCellTrait<T, Token> + ?Sized,
    Token: TokenTrait + 'a,
> {
    pub(crate) cell: &'a Cell,
    pub(crate) f: F,
    pub(crate) marker: core::marker::PhantomData<(&'a T, U, Token)>,
}
impl<
        'a,
        T: ?Sized,
        U,
        F: FnOnce(TokenGuard<'a, T, Token>) -> U,
        Token: TokenTrait,
        Cell: TokenCellTrait<T, Token>,
    > TokenMap<'a, T, U, F, Cell, Token>
{
    pub fn try_apply(self, token: &'a Token) -> Result<U, Token::ComparisonError> {
        let borrowed = self.cell.try_guard(token)?;
        Ok((self.f)(borrowed))
    }
    pub fn apply(self, token: &'a Token) -> U
    where
        Token: TokenTrait<ComparisonError = Infallible>,
    {
        let borrowed = self.cell.try_guard(token).unwrap();
        (self.f)(borrowed)
    }
}
#[must_use = "TokenMaps must be applied to do anything. Note that the closure execution will be deferred to the call-site of `apply/try_apply`"]
pub struct TokenMapMut<
    'a,
    T: ?Sized,
    U,
    F: FnOnce(TokenGuardMut<'a, T, Token>) -> U,
    Cell: TokenCellTrait<T, Token> + ?Sized,
    Token: TokenTrait + 'a,
> {
    pub(crate) cell: &'a Cell,
    pub(crate) f: F,
    pub(crate) marker: core::marker::PhantomData<(&'a T, U, Token)>,
}
impl<
        'a,
        T: ?Sized,
        U,
        F: FnOnce(TokenGuardMut<'a, T, Token>) -> U,
        Token: TokenTrait,
        Cell: TokenCellTrait<T, Token>,
    > TokenMapMut<'a, T, U, F, Cell, Token>
{
    pub fn try_apply(self, token: &'a mut Token) -> Result<U, Token::ComparisonError> {
        let borrowed = self.cell.try_guard_mut(token)?;
        Ok((self.f)(borrowed))
    }
    pub fn apply(self, token: &'a mut Token) -> U
    where
        Token: TokenTrait<ComparisonError = Infallible>,
    {
        let borrowed = self.cell.try_guard_mut(token).unwrap();
        (self.f)(borrowed)
    }
}
impl<T: ?Sized, Token: TokenTrait> TokenCell<T, Token> {}
