use token_cell::*;

generate_token!(Token, pub Token2);
fn main() {
    let mut token = Token::aquire().unwrap();
    let cell1: TokenCell<Token, _> = TokenCell::new(1);
    let cell2 = TokenCell::new(2);
    let mut token2 = Token2::aquire().unwrap();
    let mut cell3 = TokenCell::with_token(3, &token2);
    let _cell4 = TokenCell::<Token2, _>::new(4);
    *cell1.borrow_mut(&mut token) = 5;
    *cell2.borrow_mut(&mut token) = 6;
    let cell3_ref = &cell3;
    *cell3_ref.borrow_mut(&mut token2) = 7;
    let cell3_mutref = &mut cell3;
    *cell3_mutref.as_mut() = 8;
    let borrow = cell3.borrow(&token2);
    println!("{}", borrow);
}
