//! This library provides an alternative to [`ghost-cell`](https://crates.io/crates/ghost-cell) which uses concrete types instead of lifetimes for branding.
//!
//! This allows a more convenient usage, where cells and tokens can be constructed independently, with the same compile-time guarantees as [`ghost-cell`](https://crates.io/crates/ghost-cell). The trade-off for this arguably more convenient usage and arguably easier to understand branding method is that tokens, while zero-sized if made correctly, must be guaranteed to be constructable only if no other instance exists.
//!
//! To this end, this crate provides the [`generate_token`] macro, which will create a ZST which can only be constructed using [`TokenTrait::aquire`], which is generated to guarantee no other token exists before returning the token. This is done by checking a static `AtomicBool` flag, which is the only runtime cost of these tokens.
#![no_std]
use core::cell::UnsafeCell;

/// Must be implemented by any token type and ideally its only constructor.
///
/// Ideally, only a single instance of the token type should be able to exist at any point in time.
/// The easiest way to create a token is with [`generate_token`], which will create a ZST which implements [`TokenTrait`]
pub unsafe trait TokenTrait: Sized {
    fn aquire() -> Option<Self>;
}

/// Generates any number of token types for use with token cells.
///
/// These token types use a static [`AtomicBool`](std::sync::atomic::AtomicBool) to ensure unicity at any time. You may loop over aquire to obtain a spin lock, but you probably should put the token in a mutex instead.
///
/// You may generate multiple token types at once: `generate_token!(T1, pub T2)`
#[macro_export]
macro_rules! generate_token {
    ($vis: vis $id: ident) => {
        /// An auto-generated token type, initialize it with [`TokenTrait::aquire`]
        #[allow(dead_code)]
        $vis struct $id {
            do_not_initialize_manually__use_TokenTrait_aquire_instead__otherwise_I_cannot_guarantee_your_safety__so_I_am_making_this_as_obnoxious_as_possible_to_dissuade_you: (),
        }
        #[allow(non_upper_case_globals)]
        static $id: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        unsafe impl token_cell::TokenTrait for $id {
            fn aquire() -> Option<Self> {
                if $id.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    None
                } else {
                    Some($id { do_not_initialize_manually__use_TokenTrait_aquire_instead__otherwise_I_cannot_guarantee_your_safety__so_I_am_making_this_as_obnoxious_as_possible_to_dissuade_you: () })
                }
            }
        }
        impl Drop for $id {
            fn drop(&mut self) {
                $id.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
    };
    ($($vis:vis $id: ident),+) => {
        $(generate_token!($vis $id);)*
    };
}

/// A cell which requires a token for interior read/write access.
#[repr(transparent)]
pub struct TokenCell<Token: TokenTrait, T: ?Sized> {
    _marker: core::marker::PhantomData<Token>,
    inner: UnsafeCell<T>,
}

impl<Token: TokenTrait, T> TokenCell<Token, T> {
    /// Typically used to build a [`TokenCell`] when no token is immediately available, this will typically require type disambiguation.
    #[inline(always)]
    pub fn new(value: T) -> Self {
        TokenCell {
            _marker: core::default::Default::default(),
            inner: UnsafeCell::new(value),
        }
    }
    /// For convenience, when a reference to the token is available, you may use this method instead of [`TokenCell::new`],
    /// this should remove the need for you to write a turbo operator.
    #[inline(always)]
    pub fn with_token(value: T, _token: &Token) -> Self {
        TokenCell {
            _marker: core::default::Default::default(),
            inner: UnsafeCell::new(value),
        }
    }
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
    #[inline(always)]
    pub fn swap(&self, mut value: T, token: &mut Token) -> T {
        core::mem::swap(self.borrow_mut(token), &mut value);
        value
    }
    #[inline(always)]
    pub fn take(&self, token: &mut Token) -> T
    where
        T: Default,
    {
        let mut value = Default::default();
        core::mem::swap(self.borrow_mut(token), &mut value);
        value
    }
}
impl<Token: TokenTrait, T: ?Sized> TokenCell<Token, T> {
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
    #[inline(always)]
    pub fn borrow<'l>(&'l self, _: &'l Token) -> &'l T {
        unsafe { &*self.inner.get() }
    }
    #[inline(always)]
    pub fn borrow_mut<'l>(&'l self, _: &'l mut Token) -> &'l mut T {
        unsafe { &mut *self.inner.get() }
    }
    pub fn from_mut(value: &mut T) -> &mut Self {
        unsafe { core::mem::transmute(value) }
    }
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut T {
        self.inner.get()
    }
}
impl<Token: TokenTrait, S> TokenCell<Token, S> {
    pub fn as_slice_of_cells<T>(&self) -> &[TokenCell<Token, T>]
    where
        S: AsRef<[T]>,
    {
        unsafe {
            &*(<S as AsRef<[T]>>::as_ref(&*self.as_ptr()) as *const _
                as *const [TokenCell<Token, T>])
        }
    }
}
impl<Token: TokenTrait, T> AsMut<T> for TokenCell<Token, T> {
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}
impl<Token: TokenTrait, T> From<T> for TokenCell<Token, T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
unsafe impl<Token: TokenTrait, T: Send> Send for TokenCell<Token, T> {}
unsafe impl<Token: TokenTrait, T: Sync> Sync for TokenCell<Token, T> {}
