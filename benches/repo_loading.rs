use cherry_harvest::{collect_commits, git, RepoLocation};
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

const DATASET: &str = "/home/alex/data/software-sync";
fn repo_location() -> RepoLocation {
    RepoLocation::Filesystem(PathBuf::from(DATASET))
}

pub fn commit_loading(c: &mut Criterion) {
    c.bench_function("collect_commits", |b| {
        b.iter(|| {
            let repository = git::clone_or_load(&repo_location()).unwrap();
            collect_commits(&[repository]);
        })
    });
}

criterion_group!(benches, commit_loading);
criterion_main!(benches);
