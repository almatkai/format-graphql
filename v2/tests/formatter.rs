use serde_json::Value;
use tezbar_format_graphql::{format, Indentation, MAX_INPUT};
#[test]
fn pinned_prettier_corpus() {
    let corpus: Value = serde_json::from_str(include_str!("../fixtures/oracle.json")).unwrap();
    let mut mismatches = vec![];
    for (i, row) in corpus["rows"].as_array().unwrap().iter().enumerate() {
        let input = row["input"].as_str().unwrap();
        let indent = Indentation::from_value(row["indent"].as_str().unwrap()).unwrap();
        let got = format(input, indent);
        if let Some(expected) = row["output"].as_str() {
            if got.as_deref() != Ok(expected) {
                mismatches.push(format!(
                    "#{i} {} input={input:?}\nexpected={expected:?}\nactual={got:?}",
                    row["indent"]
                ));
            } else if row["idempotent"] == true {
                assert_eq!(
                    format(expected, indent).unwrap(),
                    expected,
                    "idempotence #{i}"
                );
            }
        } else if got.is_ok() {
            mismatches.push(format!(
                "#{i} accepted invalid/reference error {input:?}: {got:?}"
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}
#[test]
fn bounded_private_errors() {
    assert!(format(&"a".repeat(MAX_INPUT + 1), Indentation::Tabs)
        .unwrap_err()
        .to_string()
        .contains("limit"));
    let deep = format!("{{{}x{}}}", "f{".repeat(65), "}".repeat(65));
    assert!(format(&deep, Indentation::Spaces2).is_err());
    for forged in ["", "2x", "16", "-1", "tab "] {
        assert!(Indentation::from_value(forged).is_err());
    }
}

#[test]
fn representation_and_allocation_limits_are_explicit() {
    let gap: Value =
        serde_json::from_str(include_str!("../fixtures/representation-gap.json")).unwrap();
    for row in gap["rows"].as_array().unwrap() {
        assert_eq!(
            format(row["input"].as_str().unwrap(), Indentation::Spaces2)
                .unwrap_err()
                .to_string(),
            row["nativeError"]
        );
    }
    assert!(
        format(&format!("{{{}}}", "a ".repeat(16_385)), Indentation::Tabs)
            .unwrap_err()
            .to_string()
            .contains("token limit")
    );
    assert!(
        format(&format!("{}{{a}}", "# c\n".repeat(513)), Indentation::Tabs)
            .unwrap_err()
            .to_string()
            .contains("comment limit")
    );
    let expanded = format!(
        "{{{}f(a:\"\"\"\n{}\"\"\"){}}}",
        "f{".repeat(55),
        "x\n".repeat(2000),
        "}".repeat(55)
    );
    assert!(format(&expanded, Indentation::Spaces8)
        .unwrap_err()
        .to_string()
        .contains("output limit"));
}
