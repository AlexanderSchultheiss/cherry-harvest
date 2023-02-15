use crate::git::LineType;
use crate::{Commit, Diff};
use firestorm::{profile_fn, profile_method, profile_section};
use std::collections::{HashMap, HashSet};

pub type Similarity = f64;

pub struct DiffSimilarity<'a> {
    change_map: HashMap<&'a str, HashSet<String>>,
}

impl<'a> DiffSimilarity<'a> {
    pub fn new() -> Self {
        Self {
            change_map: HashMap::new(),
        }
    }

    pub fn change_similarity(&mut self, commit_a: &'a Commit, commit_b: &'a Commit) -> Similarity {
        profile_method!(change_similarity);
        {
            profile_section!(check_and_insert);
            if !self.change_map.contains_key(commit_a.id()) {
                self.change_map
                    .insert(commit_a.id(), Self::extract_changes(commit_a.diff()));
            }
            if !self.change_map.contains_key(commit_b.id()) {
                self.change_map
                    .insert(commit_b.id(), Self::extract_changes(commit_b.diff()));
            }
        }

        {
            profile_section!(get_and_calculate);
            let changes_a = self.change_map.get(commit_a.id()).unwrap();
            let changes_b = self.change_map.get(commit_b.id()).unwrap();
            let diff_lines_a: HashSet<&str> = commit_a.diff().diff_text().lines().collect();
            let diff_lines_b: HashSet<&str> = commit_b.diff().diff_text().lines().collect();

            {
                profile_section!(intersection_and_similarity);
                let intersection_size_changes = changes_a.intersection(changes_b).count() as f64;
                let union_size_changes = changes_a.union(changes_b).count() as f64;
                let intersection_size_diff =
                    diff_lines_a.intersection(&diff_lines_b).count() as f64;
                let union_size_diff = diff_lines_a.union(&diff_lines_b).count() as f64;

                let jaccard_changes = intersection_size_changes / union_size_changes;
                let jaccard_diff = intersection_size_diff / union_size_diff;
                (jaccard_changes + jaccard_diff) / 2.0
            }
        }
    }

    fn extract_changes(diff: &Diff) -> HashSet<String> {
        profile_fn!(extract_changes);
        let mut change_count: HashMap<String, u32> = HashMap::new();

        diff.hunks
            .iter()
            .flat_map(|h| h.body())
            .filter(|l| {
                matches!(
                    l.line_type(),
                    LineType::Addition
                        | LineType::Deletion
                        | LineType::AddEofnl
                        | LineType::DelEofnl
                )
            })
            // Append the line type prefix to the line
            .map(|l| l.line_type().char().to_string() + l.content().trim())
            .map(|change_line| {
                // We add a count to each change to distinguish between multiple occurrences of the same change
                let count = change_count.entry(change_line.clone()).or_insert(0);
                *count += 1;
                format!("{change_line}|>>{count}<<|")
            })
            .collect::<HashSet<String>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::git::IdeaPatch;
    use crate::search::methods::lsh::compare::DiffSimilarity;
    use crate::{Commit, Diff};
    use git2::Time;
    use log::{debug, LevelFilter};

    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .try_init();
    }

    fn cherry_a() -> Commit {
        Commit::new(
            "cherry_a".to_string(),
            "Some message".to_string(),
            Diff::from(IdeaPatch(CHERRY_A.to_string())),
            "author".to_string(),
            "commiter".to_string(),
            Time::new(0, 0),
        )
    }

    fn cherry_b() -> Commit {
        Commit::new(
            "cherry_b".to_string(),
            "Some message".to_string(),
            Diff::from(IdeaPatch(CHERRY_B.to_string())),
            "author".to_string(),
            "commiter".to_string(),
            Time::new(0, 0),
        )
    }

    fn pick_a() -> Commit {
        Commit::new(
            "pick_a".to_string(),
            "Some message".to_string(),
            Diff::from(IdeaPatch(PICK_A.to_string())),
            "author".to_string(),
            "commiter".to_string(),
            Time::new(0, 0),
        )
    }

    fn pick_b() -> Commit {
        Commit::new(
            "pick_b".to_string(),
            "Some message".to_string(),
            Diff::from(IdeaPatch(PICK_B.to_string())),
            "author".to_string(),
            "commiter".to_string(),
            Time::new(0, 0),
        )
    }

    fn isolated_a() -> Commit {
        Commit::new(
            "isolated_a".to_string(),
            "Some message".to_string(),
            Diff::from(IdeaPatch(ISOLATED_COMMIT_A.to_string())),
            "author".to_string(),
            "commiter".to_string(),
            Time::new(0, 0),
        )
    }

    fn isolated_b() -> Commit {
        Commit::new(
            "isolated_b".to_string(),
            "Some message".to_string(),
            Diff::from(IdeaPatch(ISOLATED_COMMIT_B.to_string())),
            "author".to_string(),
            "commiter".to_string(),
            Time::new(0, 0),
        )
    }

    #[test]
    fn debug_diff_parsing() {
        init();
        debug!("{}", cherry_a().diff());
        debug!("{}", cherry_b().diff());
        debug!("{}", pick_a().diff());
        debug!("{}", pick_b().diff());
        debug!("{}", isolated_a().diff());
        debug!("{}", isolated_b().diff());
    }

    #[test]
    fn exact_diff_max_similar() {
        init();
        const TARGET_SIMILARITY: f64 = 0.99999;
        let mut comparator = DiffSimilarity::new();
        let cherry_a = cherry_a();
        let cherry_b = cherry_b();
        let pick_a = pick_a();
        let pick_b = pick_b();
        let isolated_a = isolated_a();
        let isolated_b = isolated_b();
        assert!(comparator.change_similarity(&cherry_a, &cherry_a) > TARGET_SIMILARITY);
        assert!(comparator.change_similarity(&cherry_b, &cherry_b) > TARGET_SIMILARITY);
        assert!(comparator.change_similarity(&pick_a, &pick_a) > TARGET_SIMILARITY);
        assert!(comparator.change_similarity(&pick_b, &pick_b) > TARGET_SIMILARITY);
        assert!(comparator.change_similarity(&isolated_a, &isolated_a) > TARGET_SIMILARITY);
        assert!(comparator.change_similarity(&isolated_b, &isolated_b) > TARGET_SIMILARITY);
    }

    #[test]
    fn cherry_and_pick_similar() {
        init();
        const TARGET_SIMILARITY: f64 = 0.5;
        let cherry_a = cherry_a();
        let pick_a = pick_a();
        let cherry_b = cherry_b();
        let pick_b = pick_b();
        let mut comparator = DiffSimilarity::new();

        // assert high similarity
        assert!(comparator.change_similarity(&cherry_a, &pick_a) > TARGET_SIMILARITY);
        assert!(comparator.change_similarity(&cherry_b, &pick_b) > TARGET_SIMILARITY);

        // assert order invariance
        assert_eq!(
            comparator.change_similarity(&cherry_a, &pick_a),
            comparator.change_similarity(&pick_a, &cherry_a)
        );
        assert_eq!(
            comparator.change_similarity(&cherry_b, &pick_b),
            comparator.change_similarity(&pick_b, &cherry_b)
        );
    }

    #[test]
    fn non_cherries_not_similar() {
        init();
        const TARGET_SIMILARITY: f64 = 0.5;
        let mut comparator = DiffSimilarity::new();

        let diffs = vec![cherry_a(), pick_b(), isolated_a(), isolated_b()];

        for (id, first) in diffs.iter().enumerate() {
            for second in &diffs[(id + 1)..] {
                assert!(comparator.change_similarity(first, second) < TARGET_SIMILARITY);
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
+#[macro_use]
+extern crate log;
 
 fn main() {
-    println!("Hello, world!");
+    env_logger::init();
+
+    info!("starting up");
 
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
