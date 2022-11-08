//! By generating the ASM for this example,
//! we find that in release mode, `infallible_borrow` and `infallible_try_borrow` are equivalent.
//! However, in debug mode, the ASM instructions for panicking are still present.

use token_cell::{prelude::*, RuntimeToken};

#[no_mangle]
fn infallible_borrow(cell: &TokenCell<i32, Token>, token: &mut Token) {
    *cell.borrow_mut(token) = 1;
}
#[no_mangle]
fn infallible_try_borrow(cell: &TokenCell<i32, Token>, token: &mut Token) {
    *cell.try_borrow_mut(token).unwrap() = 1;
}
#[no_mangle]
fn fallible_try_borrow(cell: &TokenCell<i32, RuntimeToken>, token: &mut RuntimeToken) {
    *cell.try_borrow_mut(token).unwrap() = 1;
}

token_cell::unsafe_token!(Token);
fn main() {
    let mut t1 = Token::new().unwrap();
    let c1 = TokenCell::new(0, &t1);
    infallible_borrow(&c1, &mut t1);
    infallible_try_borrow(&c1, &mut t1);
    let mut t2 = RuntimeToken::new().unwrap();
    let c2 = TokenCell::new(1, &t2);
    fallible_try_borrow(&c2, &mut t2);
}
