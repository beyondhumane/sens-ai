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
    let found = review.findings(&reviewer.review(&review.prompt("English")).unwrap().answer);
    assert!(found.iter().any(|finding| finding.rule == Rule::S1 && finding.file == "src/runner.ts"), "{found:?}");

    let review = Review::of(&index, &catalog, "total() crashes on an empty list; fix it.", PLAIN);
    let found = review.findings(&reviewer.review(&review.prompt("English")).unwrap().answer);
    assert!(!found.iter().any(|finding| finding.severity == Severity::Block), "{found:?}");
}

#[test]
#[ignore]
fn a_real_reviewer_writes_in_the_person_s_language_and_still_quotes_the_diff_as_is() {
    let root = project();
    let index = sens_index::build::build(&root);
    let catalog = Catalog::of(&index);
    let review = Review::of(&index, &catalog, "Añade una forma de ejecutar tareas desde el índice.", ONE_USE);
    let found = review.findings(&Haiku::default().review(&review.prompt("Spanish as spoken in Spain")).unwrap().answer);
    let interface = found.iter().find(|finding| finding.rule == Rule::S1 && finding.file == "src/runner.ts").unwrap_or_else(|| panic!("{found:?}"));
    let spanish = [" la ", " el ", " una ", " un ", " que ", " de ", " es "];
    assert!(spanish.iter().any(|word| format!(" {} ", interface.message.to_lowercase()).contains(word)), "{}", interface.message);
}
