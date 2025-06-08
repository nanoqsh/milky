# Hello, html!

Hmm.. want some prime numbers?

```rust
fn primes() -> impl Iterator<Item = u64> {
    let mut ps = vec![];
    (2..).filter_map(move |n| {
        ps.iter().all(|p| n % p != 0).then(|| {
            ps.push(n);
            n
        })
    })
}
```
