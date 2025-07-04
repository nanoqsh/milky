## Привет, html!

Хм.. хотите немного *простых* **чисел**?

```rust
/// Generates prime numbers
fn primes() -> impl Iterator<Item = u64> {
    /*~*/
    let mut ps = vec![];
    (2..).filter_map(move |n| {
        ps.iter().all(|p| n % p != 0).then(|| {
            ps.push(n);
            n
        })
    })
}
```

Что сделать:
* Запостить изображение
* ???
* Профит!

![](post.jpg)
