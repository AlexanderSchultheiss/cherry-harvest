use cherry_harvest::RepoLocation;
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::Path;

const DATASET: &str = "/home/alex/data/cherries-one";

fn repo_location() -> RepoLocation<'static> {
    RepoLocation::Filesystem(Path::new(DATASET))
}

fn search_call() {
    let search_method = cherry_harvest::ANNMatch::default();
    let _ = cherry_harvest::search_with(&repo_location(), search_method);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ann", |b| b.iter(search_call));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
