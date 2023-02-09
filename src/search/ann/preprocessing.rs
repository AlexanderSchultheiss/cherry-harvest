use crate::error::Error;
use crate::error::ErrorKind::ANNPreprocessing;
use crate::{Commit, Diff};
use bit_vec::BitVec;
use firestorm::{profile_fn, profile_method};
use num_traits::cast;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::sync::mpsc::channel;
use std::sync::Arc;
use threadpool::ThreadPool;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Default, Hash, Clone)]
pub struct Shingle(String);

impl Display for Shingle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct ShingledText {
    shingles: Vec<Shingle>,
    arity: usize,
}

pub fn shingle_diff(diff: &Diff, arity: usize) -> ShingledText {
    ShingledText::new(diff.diff_text(), arity)
}

pub fn shingle_text(diff: &str, arity: usize) -> ShingledText {
    ShingledText::new(diff, arity)
}

fn shingle_commits_multi_threaded(commits: &[Commit], arity: usize) -> Vec<ShingledText> {
    let n_workers = 24;
    let pool = ThreadPool::new(n_workers);

    let (sender, receiver) = channel();

    commits.iter().map(|c| c.diff().clone()).for_each(|diff| {
        let sender = sender.clone();

        pool.execute(move || {
            sender.send(shingle_diff(&diff, arity)).unwrap();
        })
    });
    drop(sender);

    receiver.iter().collect()
}

fn shingle_texts(texts: &[&str], arity: usize) -> Vec<ShingledText> {
    texts
        .iter()
        .map(|text| shingle_text(&text, arity))
        .collect()
}

pub fn preprocess_commits(
    commits: &[Commit],
    arity: usize,
    signature_size: usize,
) -> Vec<Signature> {
    profile_fn!(preprocess_commits);
    let shingled_commits = shingle_commits_multi_threaded(commits, arity);

    shingles_into_signatures_multi_threaded(shingled_commits, signature_size)
}

pub fn preprocess_texts(texts: &[&str], arity: usize, signature_size: usize) -> Vec<Signature> {
    profile_fn!(preprocess_commits);
    let shingled_commits = shingle_texts(texts, arity);

    shingles_into_signatures_multi_threaded(shingled_commits, signature_size)
}

fn shingles_into_signatures_multi_threaded(
    shingled_texts: Vec<ShingledText>,
    signature_size: usize,
) -> Vec<Signature> {
    let n_workers = 24;
    let pool = ThreadPool::new(n_workers);

    let vocabulary = Arc::new(Vocabulary::build(&shingled_texts));
    let minhash = Arc::new(MinHash::new(signature_size, vocabulary.len()));

    let (sender, receiver) = channel();
    shingled_texts.into_iter().for_each(|sd| {
        let sender = sender.clone();
        let vocabulary = Arc::clone(&vocabulary);
        let minhash = Arc::clone(&minhash);
        pool.execute(move || {
            let one_hot = vocabulary.one_hot(&sd).unwrap();
            sender.send(minhash.hash_signature(&one_hot)).unwrap();
        })
    });
    drop(sender);
    receiver.iter().collect()
}

impl ShingledText {
    pub fn new(text: &str, arity: usize) -> Self {
        profile_fn!(new_shingled_text);
        let lines: Vec<&str> = text.lines().collect();
        let mut shingles = Vec::new();
        for window_position in 0..lines.len() {
            let mut shingle_lines = Vec::with_capacity(arity);
            for index in window_position..(window_position + arity) {
                let line = lines.get(index).map_or("", |x| x).trim();
                if !line.is_empty() {
                    shingle_lines.push(line);
                }
            }
            shingles.push(Shingle(shingle_lines.concat()));
        }

        ShingledText { shingles, arity }
    }
}

impl Display for ShingledText {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for shingle in &self.shingles {
            writeln!(f, "{shingle}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Vocabulary(HashMap<Shingle, usize>);

impl Vocabulary {
    pub fn build(shingled_texts: &[ShingledText]) -> Self {
        profile_fn!(build_vocabulary);
        // Filter duplicate shingles for vocabulary creation
        let mut shingles = HashSet::new();
        shingled_texts
            .iter()
            .flat_map(|sd| &sd.shingles)
            .for_each(|s| {
                if !shingles.contains(s) {
                    shingles.insert(s.clone());
                }
            });

        // The process requires shuffled assignments for the words in the vocabulary
        let mut indices: Vec<usize> = (0..shingles.len()).collect();
        indices.shuffle(&mut thread_rng());

        let mut shingle_map = HashMap::new();
        // The vocabulary assigns each shingle a random index
        shingles.into_iter().enumerate().for_each(|(i, shingle)| {
            if shingle_map.insert(shingle, indices[i]).is_some() {
                panic!("expected no conflicts!");
            }
        });

        Self(shingle_map)
    }

    pub fn one_hot(&self, shingled_diff: &ShingledText) -> Result<BitVec, Error> {
        profile_method!(one_hot);
        let mut one_hot: BitVec = BitVec::from_elem(self.0.len(), false);

        // Set values of all occurring shingles to 1
        for shingle in &shingled_diff.shingles {
            match self.0.get(shingle) {
                None => return Err(Error::new(ANNPreprocessing("Shingle in diff not part of vocabulary. Have you used it during vocabulary building?".to_string()))),
                Some(number) => {one_hot.set(*number, true);}
            }
        }

        Ok(one_hot)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub type Signature = Vec<u32>;

pub struct MinHash {
    signature_size: usize,
    data_size: usize,
    hash_vectors: Vec<Vec<usize>>,
}

impl MinHash {
    pub fn new(signature_size: usize, data_size: usize) -> Self {
        profile_fn!(new_minhash);
        // We require one hash function for each dimension in the signature
        let mut hash_vectors = Vec::with_capacity(signature_size);
        // We require one value for each word in the vocabulary, for which we want to apply MinHash
        let mut initial_vector: Vec<usize> = (0..data_size).collect();

        let mut rng = thread_rng();
        for _ in 0..signature_size {
            initial_vector.shuffle(&mut rng);
            hash_vectors.push(initial_vector.clone())
        }

        Self {
            signature_size,
            data_size,
            hash_vectors,
        }
    }

    pub fn hash_signature(&self, one_hot: &BitVec) -> Signature {
        profile_method!(hash_signature);
        assert_eq!(
            one_hot.len(),
            self.data_size,
            "the given one-hot vector's size does not match the expected data size"
        );
        let mut signature: Signature = Vec::with_capacity(self.signature_size);

        for vector in &self.hash_vectors {
            // Get the first value that maps to a 'hot' index
            // value and index are switched here on purpose, because MinHashing expects that the values
            // are incremented from lowest to highest. Thus, we assume that our shuffled vector maps
            // values to indices (technically, its the other way around)
            for (value, index) in vector.iter().enumerate() {
                if one_hot.get(*index).unwrap() {
                    signature.push(cast(value).unwrap());
                    break;
                }
            }
        }

        signature
    }
}

#[cfg(test)]
mod tests {
    use crate::git::IdeaPatch;
    use crate::search::ann::preprocessing::{
        preprocess_commits, preprocess_texts, shingle_diff, MinHash, ShingledText, Signature,
        Vocabulary,
    };
    use crate::Diff;
    use bit_vec::BitVec;
    use num_traits::abs;

    #[test]
    fn simple_shingle_arity_3() {
        let diff = Diff::from(IdeaPatch(DIFF.to_string()));
        let arity = 3;

        let shingled_diff = shingle_diff(&diff, arity);
        assert_eq!(shingled_diff.to_string(), EXPECTED_3_SHINGLE.to_string());
    }

    #[test]
    fn simple_shingle_arity_1() {
        let diff = Diff::from(IdeaPatch(DIFF.to_string()));
        let arity = 1;

        let shingled_diff = shingle_diff(&diff, arity);
        assert_eq!(shingled_diff.to_string(), EXPECTED_1_SHINGLE.to_string());
    }

    #[test]
    fn one_hot_with_only_one_diff() {
        // We expect that all values in the one-hot encoding are 1
        let shingled_diff = vec![shingle_diff(&Diff::from(IdeaPatch(DIFF.to_string())), 3)];

        let vocabulary = Vocabulary::build(&shingled_diff);
        let one_hot = vocabulary.one_hot(&shingled_diff[0]).unwrap();

        one_hot.iter().for_each(|v| assert!(v));
    }

    #[test]
    fn one_hot_with_two_words() {
        // We expect that all values in the one-hot encoding are 1
        let shingled_texts = vec![
            ShingledText::new("a\nb\nc", 2),
            ShingledText::new("b\nc\nd", 2),
        ];

        let vocabulary = Vocabulary::build(&shingled_texts);
        let one_hot_first = vocabulary.one_hot(&shingled_texts[0]).unwrap();
        let one_hot_second = vocabulary.one_hot(&shingled_texts[1]).unwrap();

        let count_first = one_hot_first.iter().filter(|v| *v).count();
        assert_eq!(count_first, 3);
        let count_second = one_hot_second.iter().filter(|v| *v).count();
        assert_eq!(count_second, 3);
        assert_eq!(vocabulary.len(), 5);

        let ones_in_intersection = one_hot_first
            .into_iter()
            .zip(one_hot_second.into_iter())
            .map(|(first, second)| first & second)
            .filter(|v| *v)
            .count();
        assert_eq!(ones_in_intersection, 1);
    }

    #[test]
    fn text_one_hot_similarity() {
        // We expect that all values in the one-hot encoding are 1
        let shingled_texts = vec![
            ShingledText::new(TEXT, 2),
            ShingledText::new(TEXT_CLOSE, 2),
            ShingledText::new(TEXT_FAR, 2),
        ];

        let vocabulary = Vocabulary::build(&shingled_texts);
        let one_hot_base = vocabulary.one_hot(&shingled_texts[0]).unwrap();
        let one_hot_close = vocabulary.one_hot(&shingled_texts[1]).unwrap();
        let one_hot_far = vocabulary.one_hot(&shingled_texts[2]).unwrap();

        let one_hot_distance =
            |s1: &BitVec, s2: &BitVec| s1.iter().zip(s2.iter()).filter(|(v1, v2)| v1 != v2).count();

        let distance_close = one_hot_distance(&one_hot_base, &one_hot_close);
        let distance_far = one_hot_distance(&one_hot_base, &one_hot_far);
        assert!(
            distance_close < distance_far,
            "{distance_close}:{distance_far}"
        );
    }

    #[test]
    fn simple_minhash_test() {
        let minhash = MinHash::new(4, 6);

        let mut one_hot_a = BitVec::from_elem(6, false);
        one_hot_a.set(0, true);
        one_hot_a.set(3, true);
        one_hot_a.set(5, true);
        let mut one_hot_b = BitVec::from_elem(6, false);
        one_hot_b.set(1, true);
        one_hot_b.set(2, true);

        let signature_a = minhash.hash_signature(&one_hot_a);
        let signature_b = minhash.hash_signature(&one_hot_b);
        let signature_a2 = minhash.hash_signature(&one_hot_a);

        assert_eq!(signature_a, signature_a2);
        assert_ne!(signature_a, signature_b);
    }

    #[test]
    fn text_signature_similarity() {
        let signatures = preprocess_texts(&[TEXT, TEXT_CLOSE, TEXT_FAR], 3, 256);

        let sig_distance = |s1: &Signature, s2: &Signature| {
            s1.iter()
                .zip(s2.iter())
                .map(|(v1, v2)| u32::abs_diff(*v1, *v2))
                .sum::<u32>()
        };

        let distance_close = sig_distance(&signatures[0], &signatures[1]);
        let distance_far = sig_distance(&signatures[0], &signatures[2]);
        assert!(
            distance_close < distance_far,
            "{distance_close}:{distance_far}"
        );
    }

    const DIFF: &str = r#"
Subject: [PATCH] feat: removed functions
---
Index: src/main.rs
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs	(revision 3d4a3d51f625a660587ec92e186a5fd458841638)
+++ b/src/main.rs	(revision 4e39e242712568e6f9f5b6ff113839603b722683)
@@ -15,18 +15,3 @@
         println!("So much!");
     }
 }
-
-fn foo() {
-    println!("foo!");
-}
"#;

    const TEXT: &str = r#"
@@ -15,18 +15,3 @@
         println!("So much!");
 }
-fn foo() {
-    println!("bar!");
-}
     }
 }
-fn foo() {
-    println!("bar!");
-}
"#;

    const TEXT_CLOSE: &str = r#"
@@ -15,18 +15,3 @@
         println!("So much!");
     }
 }
-fn foo() {
-    println!("bar!");
-}
"#;

    const TEXT_FAR: &str = r#"
@@ -1,8 +1,12 @@
 mod dev;
 mod error;
+#[macro_use]
+extern crate log;
 
 fn main() {
-    println!("Hello, world!");
+    env_logger::init();
+
+    info!("starting up");
 
     let mut x = 0;
 
@@ -12,6 +16,6 @@
     }
 
     if x > 7 {
-        println!("So much!");
+        info!("Goodbye!");
     }
 }
 
"#;

    const EXPECTED_3_SHINGLE: &str = r#"--- a/src/main.rs+++ b/src/main.rs@@ -15,18 +15,3 @@
+++ b/src/main.rs@@ -15,18 +15,3 @@println!("So much!");
@@ -15,18 +15,3 @@println!("So much!");}
println!("So much!");}}
}}-
}--fn foo() {
--fn foo() {-    println!("foo!");
-fn foo() {-    println!("foo!");-}
-    println!("foo!");-}
-}
"#;

    const EXPECTED_1_SHINGLE: &str = r#"--- a/src/main.rs
+++ b/src/main.rs
@@ -15,18 +15,3 @@
println!("So much!");
}
}
-
-fn foo() {
-    println!("foo!");
-}
"#;
}
