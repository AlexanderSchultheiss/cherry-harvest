use cherry_harvest::git::GitRepository;
use cherry_harvest::RepoLocation;
use criterion::{criterion_group, criterion_main, Criterion};

const DATASET: &str = "https://github.com/AlexanderSchultheiss/cherries-one.git";

fn repo_location() -> RepoLocation {
    RepoLocation::Server(DATASET.to_string())
}

fn search_call() {
    let search_method = cherry_harvest::TraditionalLSH::new(3, 2048, 2, 0.7);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(cherry_harvest::search_with(
        &[&GitRepository::from(repo_location())],
        search_method,
    ));
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("traditional_lsh", |b| b.iter(search_call));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
