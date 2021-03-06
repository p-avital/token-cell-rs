use ghost_cell::{GhostCell as GC, GhostToken as GT};
use token_cell::*;

fn main() {
    GhostToken::with_token(|mut t1| {
        let c1 = TokenCell::<_, GhostToken>::new(0, &t1);
        GhostToken::with_token(move |mut t2| {
            let c2 = TokenCell::<_, GhostToken>::new(1, &t2);
            println!("{}", c2.borrow_mut(&mut t2));
            println!("{}", c2.borrow_mut(&mut t1));
            println!("{}", c1.borrow_mut(&mut t2));
        })
        .unwrap();
    })
    .unwrap();
    GT::new(|mut t1| {
        let c1 = GC::new(0);
        GT::new(move |mut t2| {
            let c2 = GC::new(1);
            println!("{}", c2.borrow_mut(&mut t2));
            println!("{}", c2.borrow_mut(&mut t1));
            println!("{}", c1.borrow_mut(&mut t1));
        });
    });
}
