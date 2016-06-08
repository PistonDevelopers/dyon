/*
> rustc -O source/bench_rust/iterate.rs
> time ./iterate
*/

fn main() {
    let mut x: f64 = 0.0;
    for i in 0..100_000_000 { x = (x + 1.0).sqrt() }
    println!("x {}", x);
}
