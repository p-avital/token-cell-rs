use ghost_cell::{GhostCell as GC, GhostToken as GT};
use token_cell::{ghost::*, prelude::*};

fn main() {
    GhostToken::with_token(|mut t1| {
        let c1 = TokenCell::new(1, &t1);
        GhostToken::with_token(|mut t2| {
            let c2 = TokenCell::new(1, &t2);
            println!("{}", *c2.borrow_mut(&mut t2));
            println!("{}", *c1.borrow_mut(&mut t1));
        })
        .unwrap();
        c1.borrow_mut(&mut t1);
    })
    .unwrap();
    GT::new(move |mut t1| {
        let c1 = GC::new(1);
        GT::new(|mut t2| {
            let c2 = GC::new(1);
            println!("{}", c2.borrow_mut(&mut t2));
            println!("{}", c1.borrow_mut(&mut t1));
        });
        println!("{}", c1.borrow_mut(&mut t1));
    });
}
