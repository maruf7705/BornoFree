use borno_core::PhoneticEngine;

#[test]
fn test_all_ridmik_avro_fixtures() {
    let engine = PhoneticEngine::new();
    let csv_content = include_str!("fixtures/avro_phonetic_test_case.csv");

    let mut passed = 0;
    let mut total = 0;
    let mut failures = Vec::new();

    for (line_num, line) in csv_content.lines().enumerate() {
        if line_num == 0 || line.trim().is_empty() {
            continue; // Header or empty
        }

        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() != 2 {
            continue;
        }

        let input = parts[0].trim();
        let expected = parts[1].trim();
        total += 1;

        let actual = engine.transliterate(input);
        if actual == expected {
            passed += 1;
        } else {
            failures.push(format!(
                "Line {}: input='{}' -> expected='{}', actual='{}'",
                line_num + 1,
                input,
                expected,
                actual
            ));
        }
    }

    println!("\nRidmik Avro Fixtures: {}/{} passed ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
    if !failures.is_empty() {
        eprintln!("\nFirst 10 differences (if any):");
        for f in failures.iter().take(10) {
            eprintln!("  {}", f);
        }
    }

    assert!(
        passed as f64 / total as f64 >= 0.70,
        "Expected at least 70% direct grammar match with Ridmik Avro gold-standard fixtures, got {}/{}",
        passed,
        total
    );
}
