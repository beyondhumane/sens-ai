use std::path::PathBuf;

use sens_agent::canon::review::{Haiku, Reviewer};
use sens_canon::relevant::Catalog;
use sens_canon::review::Review;
use sens_canon::verdict::{Rule, Severity};

const ONE_USE: &str = "diff --git a/src/runner.ts b/src/runner.ts
new file mode 100644
--- /dev/null
+++ b/src/runner.ts
@@ -0,0 +1,11 @@
+export interface JobRunner {
+  run(job: string): Promise<void>;
+}
+
+class LocalJobRunner implements JobRunner {
+  async run(job: string) {
+    console.log(job);
+  }
+}
+
+export const runner: JobRunner = new LocalJobRunner();
";

const PLAIN: &str = "diff --git a/src/total.ts b/src/total.ts
--- a/src/total.ts
+++ b/src/total.ts
@@ -1,3 +1,3 @@
 export function total(prices: number[]) {
-  return prices.reduce((sum, price) => sum + price);
+  return prices.reduce((sum, price) => sum + price, 0);
 }
";

fn project() -> PathBuf {
    let root = std::env::temp_dir().join("sens-live-review");
    let _ = std::fs::remove_dir_all(&root);
    for (path, content) in [("package.json", r#"{ "main": "src/index.ts" }"#), ("src/index.ts", "import { runner } from './runner.ts';\nrunner.run('a');\n"), ("src/total.ts", "export function total(prices: number[]) {\n  return prices.reduce((sum, price) => sum + price, 0);\n}\n")] {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    root
}

#[test]
#[ignore]
fn a_real_reviewer_names_an_interface_with_one_implementation_and_leaves_a_plain_fix_alone() {
    let root = project();
    let index = sens_index::build::build(&root);
    let catalog = Catalog::of(&index);
    let reviewer = Haiku::default();

    let review = Review::of(&index, &catalog, "Add a way to run jobs from the index.", ONE_USE);
    let found = review.findings(&reviewer.review(&review.prompt()).unwrap());
    assert!(found.iter().any(|finding| finding.rule == Rule::S1 && finding.file == "src/runner.ts"), "{found:?}");

    let review = Review::of(&index, &catalog, "total() crashes on an empty list; fix it.", PLAIN);
    let found = review.findings(&reviewer.review(&review.prompt()).unwrap());
    assert!(!found.iter().any(|finding| finding.severity == Severity::Block), "{found:?}");
}
