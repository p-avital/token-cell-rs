/// Produces tokens that are also checked at runtime, ensuring that a [`TokenCell`](crate::core::TokenCell) is never accidentally used with another instance of the same token type.
#[macro_export]
macro_rules! runtime_token {
($vis: vis $id: ident) => {
    $crate::paste! {
        $vis use [<__ $id _mod__ >]::$id;
        #[allow(nonstandard_style)]
        mod [<__ $id _mod__ >] {
            use core::convert::Infallible;
            static COUNTER: $crate::atomics::AtomicU16 = $crate::atomics::AtomicU16::new(0);
            /// A small token that's also checked at runtime, ensuring that a [`TokenCell`] is never accidentally used with another instance of the same token type.
            pub struct $id(u16);
            impl $crate::core::TokenTrait for $id {
                type ConstructionError = Infallible;
                type RunError = Infallible;
                type Identifier = u16;
                type ComparisonError = $crate::macros::IdMismatch;
                type Branded<'a> = Self;
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
                        Err($crate::macros::IdMismatch {
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
    $($crate::runtime_token!($vis $id);)*
}
}

/// Produces tokens whose only identifier is their type, but is built such that only one instance of it can exist at any given time.
///
/// Looping on [`TokenTrait::new`](crate::core::TokenTrait::new) with a singleton token to access a [`TokenCell`](crate::core::TokenCell) is equivalent to using a spin-lock.
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
            /// A ZST tokens whose only identifier is their type, but is built such that only one instance of it can exist at any given time.
            ///
            /// Looping on [`TokenTrait::new`](token_cell::core::TokenTrait::new) with this type to access a [`TokenCell`](token_cell::core::TokenCell) is equivalent to using a spin-lock.
            pub struct $id(());
            impl $crate::core::TokenTrait for $id {
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
            impl ::core::ops::Drop for $id {
                fn drop(&mut self) {
                    AVAILABLE.store(true, core::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }
};
($($vis: vis $id: ident),*) => {
    $($crate::singleton_token!($vis $id);)*
}
}

/// Produces tokens whose only identifier is their type.
///
/// While unlikely, a potential misuse is constructing multiple instances of the same type and using one to access a cell constructed by another instance.
///
/// For example, if you have multiple instances of a tree that uses a single mutex to lock all of its `Arc`-ed nodes through a token built with [`unsafe_token`](crate::unsafe_token), one's token could unlock another's node without causing any errors.
#[macro_export]
macro_rules! unsafe_token {
($vis: vis $id: ident) => {
    $crate::paste! {
        $vis use [<__ $id _mod__ >]::$id;
        #[allow(nonstandard_style)]
        mod [<__ $id _mod__ >] {
            use core::convert::Infallible;
            /// A ZST token whose only identifier is its type.
            ///
            /// While unlikely, a potential misuse is constructing multiple instances of the same type and using one to access a cell constructed by another instance.
            ///
            /// For example, if you have multiple instances of a tree that uses a single mutex to lock all of its [`Arc`](alloc::sync::Arc)-ed nodes through a token built with this type.
            pub struct $id(());
            impl $crate::core::TokenTrait for $id {
                type ConstructionError = Infallible;
                type RunError = Infallible;
                type Identifier = ();
                type ComparisonError = Infallible;
                type Branded<'a> = Self;
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
    $($crate::unsafe_token!($vis $id);)*
}
}
pub use token::token;
#[cfg(any(feature = "debug", debug_assertions))]
mod token {
    pub use crate::runtime_token as token;
}
#[cfg(not(any(feature = "debug", debug_assertions)))]
mod token {
    pub use crate::unsafe_token as token;
}

/// The comparison error for runtime tokens.
#[derive(Debug, Clone, Copy)]
pub struct IdMismatch {
    /// The identifier of the token the cell was expecting.
    pub cell: u16,
    /// The identifier of the token that was used to attempt accessing the cell's contents.
    pub token: u16,
}
impl ::core::fmt::Display for IdMismatch {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
/// The construction error for [`singleton_token`]s.
#[derive(Debug, Clone, Copy)]
pub struct SingletonUnavailable;
impl ::core::fmt::Display for SingletonUnavailable {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
