use core::{cell::UnsafeCell, convert::Infallible};

use crate::core::TokenTrait;

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
