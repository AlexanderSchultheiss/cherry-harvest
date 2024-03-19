use crate::git::LineType;
use crate::{Commit, Diff};
use firestorm::{profile_fn, profile_method};
use git2::Oid;
use std::collections::{HashMap, HashSet};

pub type Similarity = f64;

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct CountedLine<'a> {
    content: &'a str,
    count: usize,
    line_type: LineType,
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
struct UncountedLine<'a> {
    content: &'a str,
    line_type: LineType,
}

#[derive(Default)]
pub struct DiffSimilarity<'a> {
    counted_lines: HashMap<Oid, HashSet<CountedLine<'a>>>,
}

impl<'a> DiffSimilarity<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the mean Jaccard similarity for the changes and the full diff text for the two
    /// given commits. Thereby, the metric accounts for the similarity of only the changes, but
    /// also takes the similarity of context lines into account, which is important in the case
    /// of very simple changes, such as insertions of empty lines.
    ///
    /// The leading and trailing whitespace of lines is ignored.
    ///
    /// Moreover, multiple occurrences of the same line are handled by concatenating a count of
    /// how often this line has been observed.
    pub fn change_similarity(&mut self, commit_a: &'a Commit, commit_b: &'a Commit) -> Similarity {
        profile_method!(change_similarity);
        self.counted_lines
            .entry(commit_a.id())
            .or_insert_with(|| Self::counted_lines(commit_a.diff()));
        self.counted_lines
            .entry(commit_b.id())
            .or_insert_with(|| Self::counted_lines(commit_b.diff()));

        let diff_lines_a = self.counted_lines.get(&commit_a.id()).unwrap();
        let diff_lines_b = self.counted_lines.get(&commit_b.id()).unwrap();
        Self::diff_similarity(diff_lines_a, diff_lines_b)
    }

    fn diff_similarity(
        diff_lines_a: &HashSet<CountedLine>,
        diff_lines_b: &HashSet<CountedLine>,
    ) -> Similarity {
        profile_method!(diff_similarity);
        let changes_a = Self::extract_changes(diff_lines_a);
        let changes_b = Self::extract_changes(diff_lines_b);

        let intersection_size_changes = changes_a.intersection(&changes_b).count() as f64;
        let union_size_changes = changes_a.union(&changes_b).count() as f64;
        let intersection_size_diff = diff_lines_a.intersection(diff_lines_b).count() as f64;
        let union_size_diff = diff_lines_a.union(diff_lines_b).count() as f64;

        let jaccard_changes = intersection_size_changes / union_size_changes;
        let jaccard_diff = intersection_size_diff / union_size_diff;
        (jaccard_changes + jaccard_diff) / 2.0
    }

    fn counted_lines(diff: &Diff) -> HashSet<CountedLine> {
        profile_fn!(extract_changes);
        let mut change_count: HashMap<UncountedLine, usize> = HashMap::new();

        diff.hunks
            .iter()
            .flat_map(|h| h.body())
            // Append the line type prefix to the line
            .map(|l| UncountedLine {
                content: l.content().trim(),
                line_type: l.line_type(),
            })
            .map(|change_line| {
                // We add a count to each change to distinguish between multiple occurrences of the same change
                let count = change_count.entry(change_line).or_insert(0);
                *count += 1;
                CountedLine {
                    content: change_line.content,
                    count: *count,
                    line_type: change_line.line_type,
                }
            })
            .collect::<HashSet<CountedLine>>()
    }

    fn extract_changes<'b>(lines: &HashSet<CountedLine<'b>>) -> HashSet<CountedLine<'b>> {
        let mut set = HashSet::new();
        lines
            .iter()
            .filter(|l| {
                matches!(
                    l.line_type,
                    LineType::Addition
                        | LineType::Deletion
                        | LineType::AddEofnl
                        | LineType::DelEofnl
                )
            })
            .for_each(|l| {
                set.insert(*l);
            });
        set
    }
}

#[cfg(test)]
mod tests {
    use crate::git::IdeaPatch;
    use crate::search::methods::lsh::compare::DiffSimilarity;
    use crate::Diff;
    use log::{debug, LevelFilter};

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .try_init();
    }

    fn cherry_a() -> Diff {
        Diff::from(IdeaPatch(CHERRY_A.to_string()))
    }

    fn cherry_b() -> Diff {
        Diff::from(IdeaPatch(CHERRY_B.to_string()))
    }

    fn pick_a() -> Diff {
        Diff::from(IdeaPatch(PICK_A.to_string()))
    }

    fn pick_b() -> Diff {
        Diff::from(IdeaPatch(PICK_B.to_string()))
    }

    fn isolated_a() -> Diff {
        Diff::from(IdeaPatch(ISOLATED_COMMIT_A.to_string()))
    }

    fn isolated_b() -> Diff {
        Diff::from(IdeaPatch(ISOLATED_COMMIT_B.to_string()))
    }

    #[test]
    fn debug_diff_parsing() {
        init();
        debug!("{}", cherry_a());
        debug!("{}", cherry_b());
        debug!("{}", pick_a());
        debug!("{}", pick_b());
        debug!("{}", isolated_a());
        debug!("{}", isolated_b());
    }

    #[test]
    fn exact_diff_max_similar() {
        init();
        const TARGET_SIMILARITY: f64 = 0.99999;
        let (c_a, c_b, p_a, p_b, i_a, i_b) = (
            cherry_a(),
            cherry_b(),
            pick_a(),
            pick_b(),
            isolated_a(),
            isolated_b(),
        );
        let cherry_a = DiffSimilarity::counted_lines(&c_a);
        let cherry_b = DiffSimilarity::counted_lines(&c_b);
        let pick_a = DiffSimilarity::counted_lines(&p_a);
        let pick_b = DiffSimilarity::counted_lines(&p_b);
        let isolated_a = DiffSimilarity::counted_lines(&i_a);
        let isolated_b = DiffSimilarity::counted_lines(&i_b);
        assert!(DiffSimilarity::diff_similarity(&cherry_a, &cherry_a) > TARGET_SIMILARITY);
        assert!(DiffSimilarity::diff_similarity(&cherry_b, &cherry_b) > TARGET_SIMILARITY);
        assert!(DiffSimilarity::diff_similarity(&pick_a, &pick_a) > TARGET_SIMILARITY);
        assert!(DiffSimilarity::diff_similarity(&pick_b, &pick_b) > TARGET_SIMILARITY);
        assert!(DiffSimilarity::diff_similarity(&isolated_a, &isolated_a) > TARGET_SIMILARITY);
        assert!(DiffSimilarity::diff_similarity(&isolated_b, &isolated_b) > TARGET_SIMILARITY);
    }

    #[test]
    fn cherry_and_pick_similar() {
        init();
        const TARGET_SIMILARITY: f64 = 0.5;
        let (c_a, c_b, p_a, p_b) = (cherry_a(), cherry_b(), pick_a(), pick_b());
        let cherry_a = DiffSimilarity::counted_lines(&c_a);
        let cherry_b = DiffSimilarity::counted_lines(&c_b);
        let pick_a = DiffSimilarity::counted_lines(&p_a);
        let pick_b = DiffSimilarity::counted_lines(&p_b);

        // assert high similarity
        assert!(DiffSimilarity::diff_similarity(&cherry_a, &pick_a) > TARGET_SIMILARITY);
        assert!(DiffSimilarity::diff_similarity(&cherry_b, &pick_b) > TARGET_SIMILARITY);

        // assert order invariance
        assert_eq!(
            DiffSimilarity::diff_similarity(&cherry_a, &pick_a),
            DiffSimilarity::diff_similarity(&pick_a, &cherry_a)
        );
        assert_eq!(
            DiffSimilarity::diff_similarity(&cherry_b, &pick_b),
            DiffSimilarity::diff_similarity(&pick_b, &cherry_b)
        );
    }

    #[test]
    fn non_cherries_not_similar() {
        init();
        const TARGET_SIMILARITY: f64 = 0.5;

        let (c_a, p_b, i_a, i_b) = (cherry_a(), pick_b(), isolated_a(), isolated_b());
        let diffs = [
            DiffSimilarity::counted_lines(&c_a),
            DiffSimilarity::counted_lines(&p_b),
            DiffSimilarity::counted_lines(&i_a),
            DiffSimilarity::counted_lines(&i_b),
        ];

        for (id, first) in diffs.iter().enumerate() {
            for second in &diffs[(id + 1)..] {
                assert!(DiffSimilarity::diff_similarity(first, second) < TARGET_SIMILARITY);
            }
        }
    }

    const CHERRY_A: &str = r#"Subject: [PATCH] feat: added logging
---
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
"#;

    const PICK_A: &str = r#"
Subject: [PATCH] feat: added logging
---
Index: src/main.rs
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs	(revision 4e39e242712568e6f9f5b6ff113839603b722683)
+++ b/src/main.rs	(revision 018a1bde4fb5e987157a6e8f07a7d378d5f19484)
@@ -1,8 +1,12 @@
        mod dev;
        mod error;
+       #[macro_use]
+       extern crate log;
        
        fn main() {
-           println!("Hello, world!");
+           env_logger::init();
+       
+           info!("starting up");
        
            let mut x = 1;
"#;

    const CHERRY_B: &str = r#"
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
-
-fn faz() {
-    bar(4);
-    println!("faz: {}", 22);
-}
-
-fn bar(x: i32) {
-    let y = 7;
-    let z = x + y;
-    println!("z: {}", z);
-}
"#;

    const PICK_B: &str = r#"
Subject: [PATCH] feat: removed functions
---
Index: src/main.rs
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs	(revision 60043f22f2c933ee8c67697466dfba2679e0307d)
+++ b/src/main.rs	(revision dd594eff3dcb36e5f4bbe47176b94f6011993c71)
@@ -15,18 +15,3 @@
         println!("So much!");
     }
 }
-
-fn foo() {
-    println!("foo!");
-}
-
-fn faz() {
-    bar(4);
-    println!("faz: {}", 22);
-}
-
-fn bar(x: i32) {
-    let y = 7;
-    let z = x + y;
-    println!("z: {}", z);
-}
"#;

    const ISOLATED_COMMIT_A: &str = r#"
Subject: [PATCH] feat: added faz
---
Index: src/main.rs
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs	(revision 1e20f7ffc9af05fac517d948404ca75ac446c792)
+++ b/src/main.rs	(revision dc51211272b62f24b955429103e5588973f0c202)
@@ -20,3 +20,7 @@
 fn foo () {
     println!("foo!");
 }
+
+fn faz() {
+    println!("faz");
+}
"#;

    const ISOLATED_COMMIT_B: &str = r#"
Subject: [PATCH] feat: added counting
---
Index: src/main.rs
IDEA additional info:
Subsystem: com.intellij.openapi.diff.impl.patch.CharsetEP
<+>UTF-8
===================================================================
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs	(revision 271b69cba8d5f62a65ffe8b5cd355a64ead1eaf4)
+++ b/src/main.rs	(revision a51792eb2dc04466bf3acbf2608225c76785e9cd)
@@ -1,7 +1,15 @@
 fn main() {
     println!("Hello, world!");
 
+    let x = 0;
+
     for i in 1..10 {
         println!("At {}", i);
+        x += i;
+    };
+
+
+    if x > 10 {
+        println!("So much!");
     }
 }
"#;

    // end of module
}
