#[macro_export]
macro_rules! runtime_token {
($vis: vis $id: ident) => {
    $crate::paste! {
        $vis use [<__ $id _mod__ >]::$id;
        #[allow(nonstandard_style)]
        mod [<__ $id _mod__ >] {
            use core::{convert::Infallible, sync::atomic::AtomicU16};
            static COUNTER: AtomicU16 = AtomicU16::new(0);
            pub struct $id(u16);
            impl $crate::core::TokenTrait for $id {
                type ConstructionError = Infallible;
                type RunError = Infallible;
                type Identifier = u16;
                type ComparisonError = $crate::macros::IdMismatch;
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

#[macro_export]
macro_rules! unsafe_token {
($vis: vis $id: ident) => {
    $crate::paste! {
        $vis use [<__ $id _mod__ >]::$id;
        #[allow(nonstandard_style)]
        mod [<__ $id _mod__ >] {
            use core::convert::Infallible;
            pub struct $id(());
            impl $crate::core::TokenTrait for $id {
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

#[derive(Debug, Clone, Copy)]
pub struct IdMismatch {
    pub cell: u16,
    pub token: u16,
}
impl ::core::fmt::Display for IdMismatch {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct SingletonUnavailable;
impl ::core::fmt::Display for SingletonUnavailable {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
