use token_cell::prelude::*;

token_cell::unsafe_token!(Token, pub Token2);

fn main() {
    let mut token = Token::new().unwrap();
    let cell1 = TokenCell::new(1, &token);
    let cell2 = TokenCell::new(2, &token);
    let mut token2 = Token2::new().unwrap();
    let mut cell3 = TokenCell::new(3, &token2);
    let _cell4 = TokenCell::new(4, &token2);
    *cell1.borrow_mut(&mut token) = 5;
    *cell2.borrow_mut(&mut token) = 6;
    let map = cell2.map_mut(|mut v| *v = 9);
    let cell3_ref = &cell3;
    *cell3_ref.borrow_mut(&mut token2) = 7;
    let cell3_mutref = &mut cell3;
    *cell3_mutref.get_mut() = 8;
    let borrow = cell3.borrow(&token2);
    map.apply(&mut token);
    println!("{}", *borrow);
}
