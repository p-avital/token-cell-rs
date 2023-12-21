use core::{cell::UnsafeCell, convert::Infallible};

use crate::core::TokenTrait;

/// The identifier for a [`GhostToken`]-based cell is its [`InvariantLifetime`]
#[derive(Clone, Copy)]
pub struct InvariantLifetime<'a>(core::marker::PhantomData<UnsafeCell<&'a ()>>);
impl<'a> InvariantLifetime<'a> {
    const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

/// A lifetime based token, inspired by `ghost_cell`.
///
/// Correct usage, where cells are only unlocked by providing a mutable reference to the token they were constructed with, will compile.
/// ```rust
/// # use token_cell::{prelude::*, ghost::GhostToken};
/// GhostToken::with_token(|mut t1| {
///     let c1 = TokenCell::new(1, &t1);
///     GhostToken::with_token(|mut t2| {
///         let c2 = TokenCell::new(1, &t2);
///         println!("{}", *c2.borrow_mut(&mut t2));
///         println!("{}", *c1.borrow_mut(&mut t1));
///     })
///     .unwrap();
///     c1.borrow_mut(&mut t1);
/// })
/// .unwrap();
/// ```
///
/// But using the wrong token with any given cell will fail at compile time.
/// ```compile_fail
///  # use token_cell::{prelude::*, ghost::GhostToken};
///  GhostToken::with_token(|mut t1| {
///      let c1 = TokenCell::new(1, &t1);
///      GhostToken::with_token(|mut t2| {
///          println!("{}", *c1.borrow_mut(&mut t2));
///      })
///      .unwrap();
///      c1.borrow_mut(&mut t1);
///  })
///  .unwrap();
///  ```
/// ```compile_fail
/// # use token_cell::{prelude::*, ghost::GhostToken};
/// GhostToken::with_token(|mut t1| {
///     let c1 = TokenCell::new(1, &t1);
///     GhostToken::with_token(|mut t2| {
///         let c2 = TokenCell::new(1, &t2);
///         println!("{}", *c2.borrow_mut(&mut t1));
///         println!("{}", *c1.borrow_mut(&mut t1));
///     })
///     .unwrap();
///     c1.borrow_mut(&mut t1);
/// })
/// .unwrap();
/// ```
pub struct GhostToken<'brand>(InvariantLifetime<'brand>);
impl<'brand> TokenTrait for GhostToken<'brand> {
    type ConstructionError = ();
    type RunError = Infallible;
    type Identifier = InvariantLifetime<'brand>;
    type ComparisonError = Infallible;
    type Branded<'a> = GhostToken<'a>;
    fn new() -> Result<Self, Self::ConstructionError> {
        Err(())
    }
    /// ```rust
    /// use token_cell::{ghost::*, prelude::*};
    /// GhostToken::with_token(|mut t1| {
    ///     let c1 = TokenCell::new(1, &t1);
    ///     GhostToken::with_token(|mut t2| {
    ///         let c2 = TokenCell::new(1, &t2);
    ///         println!("{}", *c2.borrow_mut(&mut t2));
    ///         println!("{}", *c1.borrow_mut(&mut t1));
    ///     })
    ///     .unwrap();
    ///     c1.borrow_mut(&mut t1);
    /// })
    /// .unwrap();
    /// ```
    /// ```compile_fail
    /// use token_cell::{ghost::*, prelude::*};
    /// GhostToken::with_token(|mut t1| {
    ///     let c1 = TokenCell::new(1, &t1);
    ///     GhostToken::with_token(|mut t2| {
    ///         let c2 = TokenCell::new(1, &t2);
    ///         println!("{}", *c2.borrow_mut(&mut t2));
    ///         println!("{}", *c1.borrow_mut(&mut t2));
    ///     })
    ///     .unwrap();
    ///     c1.borrow_mut(&mut t1);
    /// })
    /// .unwrap();
    /// ```
    fn with_token<R, F: for<'a> FnOnce(Self::Branded<'a>) -> R>(f: F) -> Result<R, Self::RunError> {
        Ok(f(Self(InvariantLifetime::new())))
    }

    fn identifier(&self) -> InvariantLifetime<'brand> {
        self.0
    }

    fn compare(&self, _: &InvariantLifetime<'brand>) -> Result<(), Self::ComparisonError> {
        Ok(())
    }
}
