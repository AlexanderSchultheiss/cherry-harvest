use crate::error::Error;
use crate::error::ErrorKind::ANNPreprocessing;
use crate::{Commit, Diff};
use bit_vec::BitVec;
use firestorm::{profile_fn, profile_method};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

pub type Shingle<'a> = &'a str;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct ShingledText<'a> {
    shingles: Vec<Shingle<'a>>,
    arity: usize,
}

pub fn shingle_diff(diff: &Diff, arity: usize) -> ShingledText {
    ShingledText::new(diff.diff_text(), arity)
}

pub fn shingle_text(diff: &str, arity: usize) -> ShingledText {
    ShingledText::new(diff, arity)
}

fn shingle_commits(commits: &[Commit], arity: usize) -> Vec<ShingledText> {
    commits
        .iter()
        .map(|c| shingle_diff(c.diff(), arity))
        .collect()
}

fn shingle_texts<'a>(texts: &[&'a str], arity: usize) -> Vec<ShingledText<'a>> {
    texts.iter().map(|text| shingle_text(text, arity)).collect()
}

pub fn preprocess_commits(
    commits: &[Commit],
    arity: usize,
    signature_size: usize,
) -> Vec<Signature> {
    profile_fn!(preprocess_commits);
    let shingled_commits = shingle_commits(commits, arity);

    shingles_into_signatures(shingled_commits, signature_size)
}

pub fn encode_commits_f64(commits: &[Commit], arity: usize) -> Vec<Vec<f64>> {
    profile_fn!(preprocess_commits);
    let shingled_commits = shingle_commits(commits, arity);
    let vocabulary = Vocabulary::build(&shingled_commits);
    shingled_commits
        .iter()
        .map(|s| vocabulary.encode_f64(s).unwrap())
        .collect()
}

pub fn encode_commits_u32(commits: &[Commit], arity: usize) -> Vec<Vec<u32>> {
    profile_fn!(preprocess_commits);
    let shingled_commits = shingle_commits(commits, arity);
    let vocabulary = Vocabulary::build(&shingled_commits);
    shingled_commits
        .iter()
        .map(|s| vocabulary.encode_u32(s).unwrap())
        .collect()
}

pub fn preprocess_texts(texts: &[&str], arity: usize, signature_size: usize) -> Vec<Signature> {
    profile_fn!(preprocess_commits);
    let shingled_commits = shingle_texts(texts, arity);

    shingles_into_signatures(shingled_commits, signature_size)
}

fn shingles_into_signatures(
    shingled_texts: Vec<ShingledText>,
    signature_size: usize,
) -> Vec<Signature> {
    let vocabulary = Vocabulary::build(&shingled_texts);
    let minhash = MinHash::new(signature_size, vocabulary.len());
    shingled_texts
        .iter()
        .map(|st| {
            let one_hot = vocabulary.one_hot(&st).unwrap();
            minhash.hash_signature(&one_hot)
        })
        .collect()
}

impl<'a> ShingledText<'a> {
    // pub fn new(text: &str, arity: usize) -> Self {
    //     profile_fn!(new_shingled_text);
    //     let lines: Vec<&str> = text.lines().collect();
    //     let mut shingles = Vec::new();
    //     for window_position in 0..lines.len() {
    //         let mut shingle_lines = Vec::with_capacity(arity);
    //         for index in window_position..(window_position + arity) {
    //             let line = lines.get(index).map_or("", |x| x).trim();
    //             match line.is_empty() {
    //                 // we have to treat empty lines, because otherwise a ShingledText might be completely empty
    //                 true => shingle_lines.push("\n"),
    //                 false => shingle_lines.push(line),
    //             }
    //         }
    //         shingles.push(Shingle(shingle_lines.concat()));
    //     }
    //
    //     if shingles.is_empty() {
    //         shingles.push(Shingle("EMPTY".to_string()));
    //     }
    //
    //     ShingledText { shingles, arity }
    // }
    pub fn new(text: &'a str, arity: usize) -> Self {
        profile_fn!(new_shingled_text);
        let mut shingles = Vec::new();
        let char_indices = text.char_indices().map(|(i, _)| i).collect::<Vec<usize>>();

        for (i, window_position) in char_indices.iter().enumerate() {
            // chars can take more than one index; thus, we have to index into the char_indices vector
            let index_of_end_index = i + arity;
            if index_of_end_index >= char_indices.len() {
                break;
            }
            let window_end = char_indices[index_of_end_index];

            let shingle = &text[*window_position..window_end];
            shingles.push(shingle);
        }

        if shingles.is_empty() {
            shingles.push("EMPTY");
        }

        ShingledText { shingles, arity }
    }
}

impl<'a> Display for ShingledText<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for shingle in &self.shingles {
            writeln!(f, "{shingle}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Vocabulary<'text>(HashMap<Shingle<'text>, usize>);

impl<'text> Vocabulary<'text> {
    pub fn build(shingled_texts: &'text [ShingledText]) -> Self {
        profile_fn!(build_vocabulary);
        // Filter duplicate shingles for vocabulary creation
        let mut shingles = HashSet::new();
        shingled_texts
            .iter()
            .flat_map(|sd| &sd.shingles)
            .for_each(|s| {
                if !shingles.contains(s) {
                    shingles.insert(*s);
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

    /// Encode a given shingled text by mapping each shingle to a f64 according to the vocabulary
    pub fn encode_f64(&self, shingled_text: &ShingledText) -> Result<Vec<f64>, Error> {
        let mut encoding = Vec::with_capacity(shingled_text.shingles.len());
        let norm_factor = 1.0 / self.0.len() as f64;
        for shingle in &shingled_text.shingles {
            match self.0.get(shingle) {
                None => {return Err(Error::new(
                    ANNPreprocessing("Shingle in diff not part of vocabulary. Have you used it during vocabulary building?".to_string())))}
                Some(index) => {
                    let index = *index as f64;
                    // For now, we try simple normalization to [0, 1]
                    let val = index * norm_factor;
                    encoding.push(val);
                }
            }
        }
        Ok(encoding)
    }

    /// Encode a given shingled text by mapping each shingle to an u32 according to the vocabulary
    pub fn encode_u32(&self, shingled_text: &ShingledText) -> Result<Vec<u32>, Error> {
        let mut encoding = Vec::with_capacity(shingled_text.shingles.len());
        for shingle in &shingled_text.shingles {
            match self.0.get(shingle) {
                None => {return Err(Error::new(
                    ANNPreprocessing("Shingle in diff not part of vocabulary. Have you used it during vocabulary building?".to_string())))}
                Some(index) => {
                    encoding.push(*index as u32);
                }
            }
        }
        Ok(encoding)
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
                    signature.push(value as u32);
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
    use crate::search::methods::lsh::preprocessing::{
        preprocess_texts, shingle_diff, MinHash, ShingledText, Signature, Vocabulary,
    };
    use crate::Diff;
    use bit_vec::BitVec;

    // #[test]
    // fn simple_shingle_arity_3() {
    //     let diff = Diff::from(IdeaPatch(DIFF.to_string()));
    //     let arity = 3;
    //
    //     let shingled_diff = shingle_diff(&diff, arity);
    //     assert_eq!(shingled_diff.to_string(), EXPECTED_3_SHINGLE.to_string());
    // }
    //
    // #[test]
    // fn simple_shingle_arity_1() {
    //     let diff = Diff::from(IdeaPatch(DIFF.to_string()));
    //     let arity = 1;
    //
    //     let shingled_diff = shingle_diff(&diff, arity);
    //     assert_eq!(shingled_diff.to_string(), EXPECTED_1_SHINGLE.to_string());
    // }

    #[test]
    fn one_hot_with_only_one_diff() {
        // We expect that all values in the one-hot encoding are 1
        let diff = Diff::from(IdeaPatch(DIFF.to_string()));
        let shingled_diff = vec![shingle_diff(&diff, 3)];

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
        assert_eq!(count_first, 4);
        let count_second = one_hot_second.iter().filter(|v| *v).count();
        assert_eq!(count_second, 4);
        assert_eq!(vocabulary.len(), 6);

        let ones_in_intersection = one_hot_first
            .into_iter()
            .zip(one_hot_second.into_iter())
            .map(|(first, second)| first & second)
            .filter(|v| *v)
            .count();
        assert_eq!(ones_in_intersection, 2);
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
        let signatures = preprocess_texts(&[TEXT, TEXT_CLOSE, TEXT_FAR], 3, 8);

        let sig_distance = |s1: &Signature, s2: &Signature| {
            s1.iter().zip(s2.iter()).filter(|(v1, v2)| v1 != v2).count()
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
