use crate::git::LineType;
use crate::Diff;
use firestorm::{profile_fn, profile_method, profile_section};
use std::collections::{HashMap, HashSet};

pub type Similarity = f64;

pub struct ChangeSimilarityComparator<'a> {
    change_map: HashMap<&'a Diff, HashSet<String>>,
}

impl<'a> ChangeSimilarityComparator<'a> {
    pub fn new() -> Self {
        Self {
            change_map: HashMap::new(),
        }
    }

    pub fn change_similarity(&mut self, diff_a: &'a Diff, diff_b: &'a Diff) -> Similarity {
        profile_method!(change_similarity);
        {
            profile_section!(check_and_insert);
            if !self.change_map.contains_key(diff_a) {
                self.change_map
                    .insert(diff_a, Self::extract_changes(diff_a));
            }
            if !self.change_map.contains_key(diff_b) {
                self.change_map
                    .insert(diff_b, Self::extract_changes(diff_b));
            }
        }

        {
            profile_section!(get_and_calculate);
            let changes_a = self.change_map.get(diff_a).unwrap();
            let changes_b = self.change_map.get(diff_b).unwrap();

            {
                profile_section!(intersection_and_similarity);
                let intersection_size = changes_a.intersection(changes_b).count() as f64;
                let changes_a_ratio = intersection_size / changes_a.len() as f64;
                let changes_b_ratio = intersection_size / changes_b.len() as f64;

                f64::max(changes_a_ratio, changes_b_ratio)
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
    use crate::search::methods::similar_diff::compare::ChangeSimilarityComparator;
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
        let mut comparator = ChangeSimilarityComparator::new();
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
        let mut comparator = ChangeSimilarityComparator::new();

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
        let mut comparator = ChangeSimilarityComparator::new();

        let diffs = vec![cherry_a(), pick_b(), isolated_a(), isolated_b()];

        for (id, first) in diffs.iter().enumerate() {
            for second in &diffs[(id + 1)..] {
                assert!(comparator.change_similarity(first, second) < TARGET_SIMILARITY);
            }
        }
    }

    const CHERRY_A: &str = r#"Subject: [PATCH] feat: added logging
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
 
     let mut x = 0;
 
@@ -12,6 +16,6 @@
     }
 
     if x > 7 {
-        println!("So much!");
+        info!("Goodbye!");
     }
 }
 
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
