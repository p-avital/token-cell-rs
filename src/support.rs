use core::{convert::Infallible, sync::atomic::AtomicUsize};

/// Must be implemented by any token type and ideally its only constructor.
///
/// The easiest way to create a token type is with [`generate_token`], which will create a ZST which implements [`TokenTrait`]
pub trait TokenTrait: Sized {
    type Checker: TokenChecker<Self>;
}

pub trait TokenChecker<Token> {
    type Input;
    type Error;
    fn check(&self, token: &Token) -> Result<(), Self::Error>;
    fn new(input: Self::Input) -> Self;
    fn from_ref(token: &Token) -> Self;
}

/// Generates any number of token types for use with token cells.
///
/// You may generate multiple token types at once: `generate_token!(T1, pub T2)`
#[macro_export]
macro_rules! generate_static_token {
    ($vis: vis $id: ident) => {
        /// An auto-generated token type, initialize it with [`TokenTrait::new`]
        $vis struct $id(token_cell::support::NoopChecker);
        impl $id {
            pub fn new() -> Self {
                unsafe {$id(token_cell::support::NoopChecker::new())}
            }
        }
        impl token_cell::support::TokenTrait for $id {
            type Checker = token_cell::support::NoopChecker;
        }
    };
    ($($vis:vis $id: ident),+) => {
        $(token_cell::generate_static_token!($vis $id);)*
    };
}

#[derive(Clone, Copy)]
pub struct NoopChecker(());
impl NoopChecker {
    /// # Safety
    /// Do not construct this type yourself, _really_.
    pub unsafe fn new() -> Self {
        NoopChecker(())
    }
}
impl<T> TokenChecker<T> for NoopChecker {
    type Error = Infallible;
    type Input = ();

    fn check(&self, _: &T) -> Result<(), Self::Error> {
        Ok(())
    }
    fn new(_: ()) -> Self {
        unsafe { NoopChecker::new() }
    }
    fn from_ref(_: &T) -> Self {
        unsafe { NoopChecker::new() }
    }
}

static UID: AtomicUsize = AtomicUsize::new(0);
pub struct RuntimeToken {
    id: usize,
}
impl RuntimeToken {
    pub fn new() -> Self {
        RuntimeToken {
            id: UID.fetch_add(1, core::sync::atomic::Ordering::Relaxed),
        }
    }
}
impl Default for RuntimeToken {
    fn default() -> Self {
        Self::new()
    }
}
impl TokenTrait for RuntimeToken {
    type Checker = Self;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdMismatch;
impl core::fmt::Display for IdMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("IdMismatch")
    }
}
impl TokenChecker<RuntimeToken> for RuntimeToken {
    type Error = IdMismatch;
    type Input = *const RuntimeToken;

    fn check(&self, token: &RuntimeToken) -> Result<(), Self::Error> {
        match self.id == token.id {
            true => Ok(()),
            false => Err(IdMismatch),
        }
    }
    fn new(input: Self::Input) -> Self {
        unsafe { RuntimeToken { id: (*input).id } }
    }
    fn from_ref(token: &RuntimeToken) -> Self {
        RuntimeToken { id: token.id }
    }
}
