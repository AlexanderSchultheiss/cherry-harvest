use crate::Diff;
use std::fmt::{Display, Formatter};

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct Shingle(String);

impl Display for Shingle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct ShingledDiff {
    shingles: Vec<Shingle>,
    arity: usize,
}

impl Display for ShingledDiff {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for shingle in &self.shingles {
            writeln!(f, "{shingle}")?;
        }
        Ok(())
    }
}

pub fn shingle_diff(diff: &Diff, arity: usize) -> ShingledDiff {
    let lines: Vec<&str> = diff.diff_text().lines().collect();
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

    ShingledDiff { shingles, arity }
}

#[cfg(test)]
mod tests {
    use crate::git::IdeaPatch;
    use crate::search::ann::preprocessing::shingle_diff;
    use crate::Diff;

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
