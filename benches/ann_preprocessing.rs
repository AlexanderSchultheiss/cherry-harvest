use bit_vec::BitVec;
use cherry_harvest::git::{IdeaPatch, LoadedRepository};
use cherry_harvest::search::ann::preprocessing::{
    preprocess_commits, shingle_diff, MinHash, ShingledText, Vocabulary,
};
use cherry_harvest::{collect_commits, git, Diff, RepoLocation};
use criterion::{criterion_group, criterion_main, Criterion};
use git2::BranchType;
use rand::random;
use std::path::Path;

pub fn shingle_arity_3_benchmark(c: &mut Criterion) {
    c.bench_function("shingle_arity_3", |b| {
        b.iter(|| {
            let diff = Diff::from(IdeaPatch(BENCHMARK_DIFF.to_string()));
            let arity = 3;

            shingle_diff(&diff, arity);
        })
    });
}

const DATASET: &str = "/home/alex/data/VEVOS_Simulation";
fn repo_location() -> RepoLocation<'static> {
    RepoLocation::Filesystem(Path::new(DATASET))
}

pub fn vocabulary_building(c: &mut Criterion) {
    let commits = match git::clone_or_load(&repo_location()).unwrap() {
        LoadedRepository::LocalRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Local)
        }
        LoadedRepository::RemoteRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Remote)
        }
    };
    let shingled_diffs: Vec<ShingledText> = commits
        .into_iter()
        .map(|c| shingle_diff(c.diff(), 3))
        .collect();
    c.bench_function("build_shingle_vocab", |b| {
        b.iter(|| {
            Vocabulary::build(&shingled_diffs);
        })
    });
}

pub fn minhash(c: &mut Criterion) {
    let bits_per_byte = 8;
    let num_byte = 10_000;
    let minhash = MinHash::new(256, bits_per_byte * num_byte);

    // get a random byte vector
    let mut bytes: Vec<u8> = vec![];
    for _ in 0..num_byte {
        bytes.push(random());
    }
    let bitvec = BitVec::from_bytes(&bytes);

    c.bench_function("calculate_minhash", |b| {
        b.iter(|| minhash.hash_signature(&bitvec))
    });
}

pub fn commit_preprocessing(c: &mut Criterion) {
    let commits = match git::clone_or_load(&repo_location()).unwrap() {
        LoadedRepository::LocalRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Local)
        }
        LoadedRepository::RemoteRepo { repository, .. } => {
            collect_commits(&repository, BranchType::Remote)
        }
    };
    c.bench_function("preprocess_commits", |b| {
        b.iter(|| {
            preprocess_commits(&commits, 3, 32);
        })
    });
}

criterion_group!(
    benches,
    shingle_arity_3_benchmark,
    vocabulary_building,
    minhash,
    commit_preprocessing
);
criterion_main!(benches);

const BENCHMARK_DIFF: &str = r#"const CHERRY_A: &str = r#"Subject: [PATCH] feat: added logging
---
Index: Cargo.lock
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/Cargo.lock b/Cargo.lock
--- a/Cargo.lock	(revision 64b6df22082134b29522f9ed7be2f278c0f12894)
+++ b/Cargo.lock	(revision b7d2e4b330165ae92e4442fb8ccfa067acd62d44)
@@ -2,6 +2,142 @@
 # It is not intended for manual editing.
 version = 3
 
+[[package]]
+name = "aho-corasick"
+version = "0.7.19"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "b4f55bd91a0978cbfd91c457a164bab8b4001c833b7f323132c0a4e1922dd44e"
+dependencies = [
+ "memchr",
+]
+
+[[package]]
+name = "atty"
+version = "0.2.14"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8"
+dependencies = [
+ "hermit-abi",
+ "libc",
+ "winapi",
+]
+
+[[package]]
+name = "cfg-if"
+version = "1.0.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "baf1de4339761588bc0619e3cbc0120ee582ebb74b53b4efbf79117bd2da40fd"
+
 [[package]]
 name = "cherries-one"
 version = "0.1.0"
+dependencies = [
+ "env_logger",
+ "log",
+]
+
+[[package]]
+name = "env_logger"
+version = "0.9.1"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "c90bf5f19754d10198ccb95b70664fc925bd1fc090a0fd9a6ebc54acc8cd6272"
+dependencies = [
+ "atty",
+ "humantime",
+ "log",
+ "regex",
+ "termcolor",
+]
+
+[[package]]
+name = "hermit-abi"
+version = "0.1.19"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "62b467343b94ba476dcb2500d242dadbb39557df889310ac77c5d99100aaac33"
+dependencies = [
+ "libc",
+]
+
+[[package]]
+name = "humantime"
+version = "2.1.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "9a3a5bfb195931eeb336b2a7b4d761daec841b97f947d34394601737a7bba5e4"
+
+[[package]]
+name = "libc"
+version = "0.2.137"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "fc7fcc620a3bff7cdd7a365be3376c97191aeaccc2a603e600951e452615bf89"
+
+[[package]]
+name = "log"
+version = "0.4.17"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "abb12e687cfb44aa40f41fc3978ef76448f9b6038cad6aef4259d3c095a2382e"
+dependencies = [
+ "cfg-if",
+]
+
+[[package]]
+name = "memchr"
+version = "2.5.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "2dffe52ecf27772e601905b7522cb4ef790d2cc203488bbd0e2fe85fcb74566d"
+
+[[package]]
+name = "regex"
+version = "1.6.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "4c4eb3267174b8c6c2f654116623910a0fef09c4753f8dd83db29c48a0df988b"
+dependencies = [
+ "aho-corasick",
+ "memchr",
+ "regex-syntax",
+]
+
+[[package]]
+name = "regex-syntax"
+version = "0.6.27"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "a3f87b73ce11b1619a3c6332f45341e0047173771e8b8b73f87bfeefb7b56244"
+
+[[package]]
+name = "termcolor"
+version = "1.1.3"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "bab24d30b911b2376f3a13cc2cd443142f0c81dda04c118693e35b3835757755"
+dependencies = [
+ "winapi-util",
+]
+
+[[package]]
+name = "winapi"
+version = "0.3.9"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419"
+dependencies = [
+ "winapi-i686-pc-windows-gnu",
+ "winapi-x86_64-pc-windows-gnu",
+]
+
+[[package]]
+name = "winapi-i686-pc-windows-gnu"
+version = "0.4.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6"
+
+[[package]]
+name = "winapi-util"
+version = "0.1.5"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178"
+dependencies = [
+ "winapi",
+]
+
+[[package]]
+name = "winapi-x86_64-pc-windows-gnu"
+version = "0.4.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f"
Index: Cargo.toml
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml	(revision 64b6df22082134b29522f9ed7be2f278c0f12894)
+++ b/Cargo.toml	(revision b7d2e4b330165ae92e4442fb8ccfa067acd62d44)
@@ -6,3 +6,5 @@
 # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
 
 [dependencies]
+env_logger = "0.9.1"
+log = "0.4.17"
\ No newline at end of file
Index: src/main.rs
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs	(revision 64b6df22082134b29522f9ed7be2f278c0f12894)
+++ b/src/main.rs	(revision b7d2e4b330165ae92e4442fb8ccfa067acd62d44)
@@ -1,5 +1,10 @@
+#[macro_use]
+extern crate log;
+
 fn main() {
-    println!("Hello, world!");
+    env_logger::init();
+
+    info!("starting up");
 
     let mut x = 0;
 
@@ -13,5 +18,5 @@
         println!("So much!");
     }
 
-    println!("Goodbye world!");
+    info!("Goodbye!");
 }
"#;
