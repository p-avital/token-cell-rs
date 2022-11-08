use ghost_cell::{GhostCell as GC, GhostToken as GT};
use token_cell::{ghost::*, prelude::*};

fn main() {
    GhostToken::with_token(|mut t1| {
        GhostToken::with_token(move |mut t2| {
            let c2 = TokenCell::new(1, &t2);
            println!("{}", *c2.borrow_mut(&mut t2));
            println!("{}", *c2.borrow_mut(&mut t1));
        })
        .unwrap();
    })
    .unwrap();
    GT::new(|mut _t1| {
        GT::new(move |mut t2| {
            let c2 = GC::new(1);
            println!("{}", c2.borrow_mut(&mut t2));
            // println!("{}", c2.borrow_mut(&mut _t1));
        });
    });
}
