fn main() {
    let mut x: f64 = 0.0;
    for i in 0..1_000_000_000 { x = (x + 1.0).sqrt() }
    println!("x {}", x);
}
