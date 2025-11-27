use criterion::{criterion_group, criterion_main, Criterion};
use dyon::run;

fn bench_push_array(c: &mut Criterion) {
    c.bench_function("push array", |b| b.iter(|| run_bench("source/bench/push_array.dyon")));
}

fn bench_push_link(c: &mut Criterion) {
    c.bench_function("push link", |b| b.iter(|| run_bench("source/bench/push_link.dyon")));
}

fn bench_push_link_for(c: &mut Criterion) {
    c.bench_function("push link for", |b| b.iter(|| run_bench("source/bench/push_link_for.dyon")));
}

#[cfg(all(not(target_family = "wasm"), feature = "threading"))]
fn bench_push_link_go(c: &mut Criterion) {
    c.bench_function("push link go", |b| b.iter(|| run_bench("source/bench/push_link_go.dyon")));
}

fn bench_push_str(c: &mut Criterion) {
    c.bench_function("push str", |b| b.iter(|| run_bench("source/bench/push_str.dyon")));
}

#[cfg(all(not(target_family = "wasm"), feature = "threading"))]
fn bench_push_in(c: &mut Criterion) {
    c.bench_function("push in", |b| b.iter(|| run_bench("source/bench/push_in.dyon")));
}

fn bench_add(c: &mut Criterion) {
    c.bench_function("add", |b| b.iter(|| run_bench("source/bench/add.dyon")));
}

fn bench_add_n(c: &mut Criterion) {
    c.bench_function("add n", |b| b.iter(|| run_bench("source/bench/add_n.dyon")));
}

fn bench_sum(c: &mut Criterion) {
    c.bench_function("sum", |b| b.iter(|| run_bench("source/bench/sum.dyon")));
}

fn bench_main(c: &mut Criterion) {
    c.bench_function("main", |b| b.iter(|| run_bench("source/bench/main.dyon")));
}

fn bench_array(c: &mut Criterion) {
    c.bench_function("array", |b| b.iter(|| run_bench("source/bench/array.dyon")));
}

fn bench_object(c: &mut Criterion) {
    c.bench_function("object", |b| b.iter(|| run_bench("source/bench/object.dyon")));
}

fn bench_call(c: &mut Criterion) {
    c.bench_function("call", |b| b.iter(|| run_bench("source/bench/call.dyon")));
}

fn bench_n_body(c: &mut Criterion) {
    c.bench_function("n body", |b| b.iter(|| run_bench("source/bench/n_body.dyon")));
}

fn bench_len(c: &mut Criterion) {
    c.bench_function("len", |b| b.iter(|| run_bench("source/bench/len.dyon")));
}

fn bench_min_fn(c: &mut Criterion) {
    c.bench_function("min fn", |b| b.iter(|| run_bench("source/bench/min_fn.dyon")));
}

fn bench_min(c: &mut Criterion) {
    c.bench_function("min", |b| b.iter(|| run_bench("source/bench/min.dyon")));
}

fn bench_primes(c: &mut Criterion) {
    c.bench_function("primes", |b| b.iter(|| run_bench("source/bench/primes.dyon")));
}

fn bench_primes_trad(c: &mut Criterion) {
    c.bench_function("primes trad", |b| b.iter(|| run_bench("source/bench/primes_trad.dyon")));
}

fn bench_threads_no_go(c: &mut Criterion) {
    c.bench_function("threads no go", |b| b.iter(|| run_bench("source/bench/threads_no_go.dyon")));
}

#[cfg(all(not(target_family = "wasm"), feature = "threading"))]
fn bench_threads_go(c: &mut Criterion) {
    c.bench_function("threads go", |b| b.iter(|| run_bench("source/bench/threads_go.dyon")));
}

criterion_group!(benches,
    bench_push_array,
    bench_push_link,
    bench_push_link_for,
    bench_push_link_go,
    bench_push_str,
    bench_push_in,
    bench_add,
    bench_add_n,
    bench_sum,
    bench_main,
    bench_array,
    bench_object,
    bench_call,
    bench_n_body,
    bench_len,
    bench_min_fn,
    bench_min,
    bench_primes,
    bench_primes_trad,
    bench_threads_no_go,
    bench_threads_go,
);
criterion_main!(benches);

fn run_bench(source: &str) {
    run(source).unwrap_or_else(|err| panic!("{}", err));
}
