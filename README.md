![Crates.io Version](https://img.shields.io/crates/v/token-cell)
![docs.rs](https://img.shields.io/docsrs/token-cell)
![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/token-cell)

# Decoupling ownership from mutability

[`token-cell`] is centered on the idea of separating ownership from mutability; much like [`ghost-cell`] which inspired it, and others such as [`qcell`] or [`singleton-cell`].

This is achieved is through a "lock and key" philosophy: an instance of `Token` acts as the key to any number of `TokenCell<T, Token>` instances, each of which owns a distinct value of type `T`: to gain mutable/immutable access to a cell's value, you must prove that you have the same kind of access to the token that acts as the lock.

<details><summary><h1>Why would you want to decouple ownership from mutability?</h1></summary>

A common representation for graphs is to have each node hold a shared reference to each of its neighbours.

In languages like C++ or Python, this is easy enough as you can gain mutable access without proving anything; and if you need that graph shared between threads, you'll just pinkie-swear that you'll look at the graph once you've gotten hold of a mutex (hopefully you'll even have some docs about that).

In Rust, it's a bit harder: let's get there incrementally:

```rust
struct Graph<T> {
	nodes: Vec<Arc<Node<T>>>,
}
struct Node<T> {
	value: T,
	neighbours: Vec<Weak<Node<T>>>,
}

fn add_fully_connected_node(graph: &mut Graph<T>, value: T) {
    let node = Arc::new_cyclic(|weak| Node {
        value,
        neighbours: graph
            .nodes
            .iter_mut()
            .map(|neighbour| {
                Arc::get_mut(neighbour)
                    .unwrap() // This crashes on the third call
                    .neighbours
                    .push(weak.clone());
                Arc::downgrade(neighbour)
            })
            .collect(),
    });
    graph.nodes.push(node);
}
```

That's not _too_ bad, we've got a bunch of nodes, and each node has a list of neighbours; we even prevent ownership cycles by using `Weak` there, but from the moment a node has neighbours, we can't mutate that node any longer, and our graph is now useless :(

Now, what's the first thing you learn to do in Rust when you want to mutate the contents of an `Arc`? Put a `Mutex` in it!

```rust
struct Graph<T> {
	nodes: Vec<Arc<Node<T>>>,
}
struct Node<T>(Mutex<NodeInner<T>>);
struct NodeInner<T> {
	value: T,
	neighbours: Vec<Weak<Node<T>>>,
}

fn add_fully_connected_node<T>(graph: &mut Graph<T>, value: T) {
    let node = Arc::new_cyclic(|weak| {
        Node(Mutex::new(NodeInner {
            value,
            neighbours: graph
                .nodes
                .iter()
                .map(|neighbour| {
                    neighbour.0.lock().unwrap().neighbours.push(weak.clone());
                    Arc::downgrade(neighbour)
                })
                .collect(),
        }))
    });
    graph.nodes.push(node);
}
```

But now, we've got a couple of problems:
1. Mutexes aren't free; they _can_ be surprisingly fast compared to their reputation, but there _is_ some overhead involved.
2. Even if you don't care about performance, you've just made it ridiculously easy for you to deadlock your program: if you do something recursive on your graph, and if you're not unlocking _before_ going to explore neighbours, you're likely to try to re-enter the same lock again; and most `Mutex` implementations aren't re-entrant, so you'll be there a while...

But we can just to what C++ and Python do, just access things willy-nilly!

```rust
struct Graph<T> {
	nodes: Vec<Arc<Node<T>>>,
}
struct Node<T>(UnsafeCell<NodeInner<T>>);
struct NodeInner<T> {
	value: T,
	neighbours: Vec<Weak<Node<T>>>,
}
fn add_fully_connected_node<T>(graph: &mut Graph<T>, value: T) {
    let node = Arc::new_cyclic(|weak| {
        Node(UnsafeCell::new(NodeInner {
            value,
            neighbours: graph
                .nodes
                .iter()
                .map(|neighbour| {
                    unsafe { &mut *neighbour.0.get() }
                        .neighbours
                        .push(weak.clone());
                    Arc::downgrade(neighbour)
                })
                .collect(),
        }))
    });
    graph.nodes.push(node);
}
```

Just make sure you only use `Node`s when you'd be allowed to use `Graph`, maybe by locking a `Mutex`?

The problem is that you're now left to manually managing `UnsafeCell`s everywhere, and that's kind of an invite to UB.

That's where `TokenCell` comes in!

```rust
token_cell::unsafe_token!(
    /// A new token type dedicated to our graph type.
    MyToken
);

struct Graph<T> {
	nodes: Vec<Arc<Node<T>>>,
	token: MyToken,
}
struct Node<T>(TokenCell<NodeInner<T>, MyToken>);
struct NodeInner<T> {
	value: T,
	neighbours: Vec<Weak<Node<T>>>,
}

fn add_fully_connected_node<T>(graph: &mut Graph<T>, value: T) {
    let node = Arc::new_cyclic(|weak| {
        let Graph { nodes, token } = graph;
        let value = NodeInner {
            value,
            neighbours: nodes
                .iter()
                .map(|neighbour| {
                    unsafe { neighbour.0.borrow_mut(token) }
                        .neighbours
                        .push(weak.clone());
                    Arc::downgrade(neighbour)
                })
                .collect(),
        };
        Node(TokenCell::new(value, token))
    });
    graph.nodes.push(node);
}
```

Now, we can (and must) provide proof that we have mutability of the graph to mutate its nodes!

</details>

# Token types

[`token-cell`] provides you with various kinds of tokens or token generators to suit your every needs!

Token generators allow you to generate your own token types, and it's generally advised to have distinct token types for each purpose (if your data structure uses `TokenCell` internally, but exposes the token to the user, it may be wise to make that a generic parameter of that data structure).

## `runtime_token`
The `runtime_token` macro can be used to generate token types whose instances are also checked at runtime:
- When you construct a token, it will obtain an identifier from an atomic counter.
- When you construct a `TokenCell` with a runtime token, that cell will remember the token's identifier.
- When you attempt to borrow the cell's content with your token, the cell will verify that you haven't mistakenly used another instance of the same token type to do so.
    - If it finds that you have, it will yield an error: **do not ignore it**, it indicates that you've used the wrong key for your lock.
    - Note that unless you're using a `u64` based runtime token (using `runtime_token!(MyToken: u64)`, `u16` is the default if no size is specified), that atomic counter may overflow and wrap around once your program generates more tokens than the identifier size would be able to count. Should this happen, there may be multiple token instances that the `TokenCell` cannot distinguish, and would then be unable to yield an error for:
        - This is why non-`u64` runtime token only allow you to use the `unsafe` access methods defined in `UnsafeTokenCellTrait`: this overflow possibility means that you _could_ (though are unlikely to) obtain two token that allow you to access a same `TokenCell`, and doing so would trigger UB.
        - Conversely, `u64`-backed runtime token are effectively guaranteed to be unique: you'd need 120 years to overflow the counter even if all you did for those 120 years was increment the atomic counter 5 billion times per second, which even modern hardware can't hope to achieve. This is why `token-cell` exports `RuntimeToken`: while it puts all the token verification at runtime rather than compile-time, that single token type can be sufficient for all your needs at once.

## `unsafe_token`

An `unsafe_token` is effectively equivalent to a `runtime_token` backed by a `u0`: because there only exists one identifier, every instance is indistinguishable.

While this makes it the most dangerous token in `token-cell`, that also makes it the cheapest by far: it has _no overhead_ in optimized builds.

It has the same safety invariants as `runtime_token`, only it has no way to detect if you've broken them. But there are still patterns where that invariant is easy to hold: used internally in a data structure that doesn't expose it, there's no reason a cell would ever be exposed to an incorrect instance of its token.

## `GhostToken`

`GhostToken` doesn't let you generate alternative types because each instance of `GhostToken` is statically guaranteed to be the only instance of that type thanks to branding by its lifetime.

`GhostToken` therefore has no runtime overhead, in the same way as `unsafe_token`.

It is however much less flexible, as a `GhostToken<'a>` cannot have a `'static` lifetime (by definition), meaning it can be difficult to use in multithreaded or async scenarios.

But because each `GhostToken` instance is guaranteed to be the only one of its type, you can use the safe access methods defined in `TokenCellTrait` when interacting with a `TokenCell`, no more `unsafe` blocks!

## `singleton_token`

A `singleton_token` is a token type that can only be instantiated by toggling a global atomic flag. This means that there's only ever one instance of that token type, and that by proxy all instances are equivalent.

Singletons are generally a bad pattern to use, but if your program already has global state, why not? You get a zero-sized token that's entirely safe!

You can even use that token's constructor as a spin lock, if for some reason you like locks that do undue work :)

# Why use `unsafe_token` or small `runtime_token` types if I'm gonna type `unsafe` everywhere I access them anyway? Shouldn't I just use `UnsafeCell` at this point?

While accessing a `TokenCell` with a "weak" token is still `unsafe` (mostly because treating it as fully `safe` would be a lie that we call "unsound" in the Rust jargon), it's _a lot less_ unsafe: `UnsafeCell` lets you do whatever you want, but you still have to uphold all of Rust's guarantees manually.

With `UnsafeCell`, you'll have to `unsafe impl Send/Sync` your types if you want to use them in multithreaded contexts, which means you'll be on the hook for accessing them only when you're allowed to.

With `TokenCell`, as long as you don't use the wrong token with the wrong cell, you're good. If your codebase imposes `SAFETY` comments, `// SAFETY: this token instance is the same as used to construct the cell` is all you need: there's no other invariant to uphold for Rust's safety invariants to be upheld _for you_.

You can even do this in multiple steps: start with a `runtime_token` to make double check that you're actually respecting that invariant, and once you're very sure, switch to `unsafe_token` to get disable those checks and run like the wind!





[`token-cell`]: https://crates.io/crates/token-cell
[`ghost-cell`]: https://crates.io/crates/ghost-cell
[`qcell`]: https://crates.io/crates/qcell
[`singleton-cell`]: https://crates.io/crates/singleton-cell