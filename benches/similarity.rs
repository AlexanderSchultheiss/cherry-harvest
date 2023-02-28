use cherry_harvest::search::methods::lsh::DiffSimilarity;
use cherry_harvest::{collect_commits, git, Commit, RepoLocation};
use criterion::{criterion_group, criterion_main, Criterion};

const DATASET: &str = "https://github.com/AlexanderSchultheiss/cherries-one.git";
fn repo_location() -> RepoLocation {
    RepoLocation::Server(DATASET.to_string())
}

pub fn diff_similarity(c: &mut Criterion) {
    let repository = git::clone_or_load(&repo_location()).unwrap();
    let commits = collect_commits(&[repository]);
    let commits: Vec<Commit> = commits.into_iter().collect();
    let mut comparator = DiffSimilarity::new();
    c.bench_function("diff_similarity", |b| {
        b.iter(|| {
            for (id, commit_a) in commits.iter().enumerate() {
                for commit_b in &commits[id..] {
                    comparator.change_similarity(commit_a, commit_b);
                }
            }
        })
    });
}

criterion_group!(benches, diff_similarity);
criterion_main!(benches);
