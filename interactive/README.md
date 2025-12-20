# Dyon-Interactive
A library for interactive coding with the Piston game engine

```dyon no_run
fn main() {
    set_window(size: (120, 120))
    ~ draw_list := []
    loop {
        if !next_event() {break}
        if render() {
            clear(color: #ffffff)

            rectangle(color: #ff0000, corner: (10, 10), size: (100, 100))

            draw(draw_list)
            clear(mut draw_list)
        }
    }
}
```

[DYON-API](./DYON-API.md) | [LICENSE-APACHE](./LICENSE-APACHE) | [LICENSE-MIT](LICENSE-MIT)

### Introduction

Install: `cargo install piston-dyon_interactive --example dyongame`

Run script: `dyongame <file.dyon>`

![snake](../images/snake.png)

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
