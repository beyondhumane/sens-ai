use std::collections::BTreeSet;

use sens_index::build;
use sens_index::testfile::is_test_file;

use crate::verdict::{Change, Finding, Rule, Severity};

const CHECKS: [&str; 15] = ["assert", "expect(", ".should", "t.Error", "t.Fatal", "Assert.", "verify(", "XCTAssert", "shouldBe", "should.", "Should -", "expect_", "@test", "Expect.", "expectEqual"];

fn spanned<'a>(lines: &[&'a str], unit: &sens_index::fingerprint::Unit) -> Vec<&'a str> {
    lines.iter().skip(unit.start_line.saturating_sub(1) as usize).take((unit.end_line + 1 - unit.start_line) as usize).copied().collect()
}

fn tested_names(path: &str, source: &str) -> BTreeSet<String> {
    let whole_file = is_test_file(path);
    let lines: Vec<&str> = source.lines().collect();
    build::analyze(path, source)
        .into_iter()
        .filter(|unit| unit.whole && unit.test && (whole_file || !unit.name.contains('.')))
        .filter(|unit| checks(&spanned(&lines, unit).join("\n")) > 0)
        .map(|unit| unit.name)
        .collect()
}

fn tested_text(path: &str, source: &str) -> String {
    if is_test_file(path) {
        return source.to_string();
    }
    let lines: Vec<&str> = source.lines().collect();
    build::analyze(path, source)
        .iter()
        .filter(|unit| unit.whole && unit.test)
        .flat_map(|unit| spanned(&lines, unit))
        .collect::<Vec<_>>()
        .join("\n")
}

fn checks(text: &str) -> usize {
    CHECKS.iter().map(|check| text.matches(check).count()).sum()
}

fn ask(change: &Change, line: u32, message: String, key: String) -> Finding {
    Finding { rule: Rule::R8, severity: Severity::Ask, file: change.path.clone(), line, message, target: None, key }
}

pub fn findings(change: &Change) -> Vec<Finding> {
    let Some(before) = change.before.as_deref() else {
        return Vec::new();
    };
    let had = tested_names(&change.path, before);
    let Some(after) = change.after.as_deref() else {
        if is_test_file(&change.path) || !had.is_empty() {
            return vec![ask(
                change,
                1,
                format!("Deleting {} removes tests. The person has to approve that; change the code so the tests still pass, unless the person asked for it.", change.path),
                format!("R8:{}", change.path),
            )];
        }
        return Vec::new();
    };
    let has = tested_names(&change.path, after);
    let mut found: Vec<Finding> = had
        .difference(&has)
        .map(|name| {
            ask(
                change,
                1,
                format!("This change removes the test `{name}` from {}. The person has to approve that; keep the test and make the code pass it.", change.path),
                format!("R8:{}:{name}", change.path),
            )
        })
        .collect();
    let (then, now) = (checks(&tested_text(&change.path, before)), checks(&tested_text(&change.path, after)));
    if now < then && found.is_empty() {
        found.push(ask(
            change,
            1,
            format!("The tests in {} go from {then} checks to {now}. The person has to approve weaker tests; make the code pass them instead.", change.path),
            format!("R8:{}:checks", change.path),
        ));
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "import { test } from 'node:test';\nimport assert from 'node:assert/strict';\n\ntest('adds', () => {\n  assert.equal(add(1), 2);\n  assert.equal(add(2), 3);\n});\n";

    fn change(path: &str, before: &str, after: Option<&str>) -> Change {
        Change { path: path.into(), before: Some(before.into()), after: after.map(str::to_string) }
    }

    #[test]
    fn deleting_a_test_file_asks_the_person() {
        let found = findings(&change("test/add.test.ts", SPEC, None));
        assert_eq!((found[0].rule, found[0].severity), (Rule::R8, Severity::Ask));
        assert!(findings(&change("src/add.ts", "export const a = 1;\n", None)).is_empty());
    }

    #[test]
    fn fewer_checks_in_a_test_file_ask_the_person_and_more_do_not() {
        let weaker = SPEC.replace("  assert.equal(add(2), 3);\n", "");
        assert_eq!(findings(&change("test/add.test.ts", SPEC, Some(&weaker))).len(), 1);
        let stronger = SPEC.replace("});", "  assert.equal(add(3), 4);\n});");
        assert!(findings(&change("test/add.test.ts", SPEC, Some(&stronger))).is_empty());
    }

    #[test]
    fn a_helper_in_a_test_file_that_checks_nothing_is_not_a_test() {
        let with_helper = format!("const shown = (output: string) => output.replace(/\\u00a0/g, \" \");\n\n{SPEC}");
        assert!(findings(&change("test/add.test.ts", &with_helper, Some(SPEC))).is_empty());
        let without_test = with_helper.replace("test('adds', () => {\n  assert.equal(add(1), 2);\n  assert.equal(add(2), 3);\n});\n", "");
        assert!(!findings(&change("test/add.test.ts", &with_helper, Some(&without_test))).is_empty());
    }

    #[test]
    fn a_test_removed_in_a_language_read_by_shape_asks_the_person() {
        let before = "import XCTest\n\nfinal class ShelfTests: XCTestCase {\n    func testSize() {\n        XCTAssertEqual(Shelf().size(), 1)\n    }\n\n    func testEmpty() {\n        XCTAssertTrue(Shelf().isEmpty)\n    }\n}\n";
        let after = before.replace("\n    func testEmpty() {\n        XCTAssertTrue(Shelf().isEmpty)\n    }\n", "");
        let found = findings(&change("Tests/ShelfTests.swift", before, Some(&after)));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].message.contains("`ShelfTests.testEmpty`"), "{}", found[0].message);
    }

    #[test]
    fn a_rust_test_removed_from_a_source_file_asks_the_person() {
        let before = "pub fn add(a: u8) -> u8 {\n    a + 1\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn adds() {\n        assert_eq!(add(1), 2);\n    }\n\n    #[test]\n    fn adds_again() {\n        assert_eq!(add(2), 3);\n    }\n}\n";
        let after = before.replace("\n    #[test]\n    fn adds_again() {\n        assert_eq!(add(2), 3);\n    }\n", "");
        let found = findings(&change("src/lib.rs", before, Some(&after)));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].message.contains("`adds_again`"));
    }
}
