use token_cell::prelude::*;

token_cell::unsafe_token!(
    /// A first, private token type.
    Token,
    /// A second, public token type.
    pub Token2
);

fn main() {
    let mut token1 = Token::new();
    let cell1 = TokenCell::new(1, &token1);
    let cell2 = TokenCell::new(2, &token1);
    let mut token2 = Token2::new();
    let mut cell3 = TokenCell::new(3, &token2);
    let _cell4 = TokenCell::new(4, &token2);
    unsafe { *cell1.borrow_mut(&mut token1) = 5 };
    unsafe { *cell2.borrow_mut(&mut token1) = 6 };
    let map = unsafe { cell2.map_mut(|mut v| *v = 9) };
    let cell3_ref = &cell3;
    unsafe { *cell3_ref.borrow_mut(&mut token2) = 7 };
    let cell3_mutref = &mut cell3;
    *cell3_mutref.get_mut() = 8;
    let borrow = unsafe { cell3.borrow(&token2) };
    map.apply(&mut token1);
    println!("{}", *borrow);

    let mut token3 = Token::new();
    // SAFETY: This is _technically_ safe, as the mutable borrows on `cell1` created with `token1` and `token2` never co-exist (which would trigger undefined behaviour).
    unsafe { *cell1.borrow_mut(&mut token3) = 8 };

    // This does go against `token_cell`'s philosophy of "To each cell their (single) key", but the actual safety invariant is that
    // a cell must never be borrowed by multiple instances of a same token _at the same time_.
    // Upholding the easier to track contract that a cell is only ever interacted with using the token instance that was used to construct it
    // naturally guarantees that invariant.

    // See the README's "`TokenCell`'s exact safety contract" section for more details.
}
