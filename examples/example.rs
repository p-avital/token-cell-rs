use token_cell::*;

generate_static_token!(Token, pub Token2);
fn main() {
    let mut token = Token::new();
    let cell1 = TokenCell::new(1);
    let cell2 = TokenCell::new(2);
    let mut token2 = Token2::new();
    let mut cell3 = TokenCell::new(3);
    let _cell4: TokenCell<Token2, _> = TokenCell::new(4);
    *cell1.borrow_mut(&mut token) = 5;
    *cell2.borrow_mut(&mut token) = 6;
    let cell3_ref = &cell3;
    *cell3_ref.borrow_mut(&mut token2) = 7;
    let cell3_mutref = &mut cell3;
    *cell3_mutref.as_mut() = 8;
    let borrow = cell3.borrow(&token2);
    println!("{}", borrow);
}
