use crate::error::Error;
use crate::error::ErrorKind::ANNPreprocessing;
use crate::Diff;
use bit_vec::BitVec;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Default, Hash)]
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

impl ShingledText {
    pub fn new(text: &str, arity: usize) -> Self {
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

pub fn shingle_diff(diff: &Diff, arity: usize) -> ShingledText {
    ShingledText::new(diff.diff_text(), arity)
}

#[derive(Debug)]
pub struct Vocabulary<'a>(HashMap<&'a Shingle, usize>);

impl<'a> Vocabulary<'a> {
    pub fn build(shingled_texts: &'a [ShingledText]) -> Self {
        // Filter duplicate shingles for vocabulary creation
        let shingles: HashSet<&Shingle> =
            shingled_texts.iter().flat_map(|sd| &sd.shingles).collect();

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
}

pub type Signature = Vec<usize>;

pub struct MinHash {
    signature_size: usize,
    data_size: usize,
    hash_vectors: Vec<Vec<usize>>,
}

impl MinHash {
    pub fn new(signature_size: usize, data_size: usize) -> Self {
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
                    signature.push(value);
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
    use crate::search::ann::preprocessing::{shingle_diff, MinHash, ShingledText, Vocabulary};
    use crate::Diff;
    use bit_vec::BitVec;

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

        let ones_in_intersection = one_hot_first
            .into_iter()
            .zip(one_hot_second.into_iter())
            .map(|(first, second)| first & second)
            .filter(|v| *v)
            .count();
        assert_eq!(ones_in_intersection, 1);
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
