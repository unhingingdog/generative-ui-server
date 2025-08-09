//! Performance tests for the JSONBalancer.

use std::time::Instant;
use telomere_json::JSONBalancer;

/// Generates a deeply nested JSON object string for performance testing.
///
/// The structure is `{"payload", "next": {"payload", "next": ...}}`.
///
/// # Arguments
/// * `depth` - The number of levels to nest.
///
/// # Returns
/// A tuple containing the generated incomplete JSON string and the expected closing string.
fn generate_deeply_nested_json(depth: usize) -> (String, String) {
    if depth == 0 {
        // A bit of a nonsensical case, but handle it.
        return ("".to_string(), "".to_string());
    }

    // The payload for each level of the object.
    // Note the JSON-escaped backslash and unicode sequence.
    let payload = r#""int":1,"literal":true,"escape":"\\","arr":[null]"#;

    let mut open_str = String::new();
    // Build the nested structure from the outside in.
    for i in 0..depth {
        open_str.push('{');
        open_str.push_str(payload);
        // Add the "next" key until the final level.
        if i < depth - 1 {
            open_str.push_str(r#","next":"#);
        }
    }

    // The expected completion is simply `depth` closing braces.
    let expected_completion = "}".repeat(depth);

    (open_str, expected_completion)
}

#[test]
fn perf_deeply_nested_object_5_levels() {
    const DEPTH: usize = 5;
    let (json_string, expected) = generate_deeply_nested_json(DEPTH);

    let mut balancer = JSONBalancer::new();

    let start = Instant::now();
    let result = balancer.process_delta(&json_string);
    let duration = start.elapsed();

    println!(
        "PERF: Processed {} levels of nested JSON in {:?}",
        DEPTH, duration
    );

    // 1. Functionally, it must produce the correct closing string.
    assert_eq!(result, Ok(expected));

    // 2. As a non-functional sanity check, ensure it's very fast.
    //    On a typical machine, this should be well under a millisecond.
    //    This assertion prevents performance from regressing unnoticed.
    assert!(
        duration.as_millis() < 50,
        "Performance test took too long: {:?}. This may indicate a performance regression.",
        duration
    );
}

#[test]
fn perf_deeply_nested_object_100_levels() {
    const DEPTH: usize = 100;
    let (json_string, expected) = generate_deeply_nested_json(DEPTH);

    let mut balancer = JSONBalancer::new();

    let start = Instant::now();
    let result = balancer.process_delta(&json_string);
    let duration = start.elapsed();

    println!(
        "PERF: Processed {} levels of nested JSON in {:?}",
        DEPTH, duration
    );

    // 1. Functionally, it must produce the correct closing string.
    assert_eq!(result, Ok(expected));

    // 2. As a non-functional sanity check, ensure it's very fast.
    //    On a typical machine, this should be well under a millisecond.
    //    This assertion prevents performance from regressing unnoticed.
    assert!(
        duration.as_millis() < 50,
        "Performance test took too long: {:?}. This may indicate a performance regression.",
        duration
    );
}

#[test]
fn perf_very_deeply_nested_object_1000_levels() {
    const DEPTH: usize = 1000;
    let (json_string, expected) = generate_deeply_nested_json(DEPTH);

    let mut balancer = JSONBalancer::new();

    let start = Instant::now();
    let result = balancer.process_delta(&json_string);
    let duration = start.elapsed();

    println!(
        "PERF: Processed {} levels of nested JSON in {:?}",
        DEPTH, duration
    );

    // 1. Functional check
    assert_eq!(result, Ok(expected));

    // 2. Performance check (with a more lenient time limit for 10x the depth)
    assert!(
        duration.as_millis() < 100,
        "Performance test took too long: {:?}. This may indicate a performance regression.",
        duration
    );
}

#[test]
fn perf_very_deeply_nested_object_100_000_levels() {
    const DEPTH: usize = 100_000;
    let (json_string, expected) = generate_deeply_nested_json(DEPTH);

    let mut balancer = JSONBalancer::new();

    let start = Instant::now();
    let result = balancer.process_delta(&json_string);
    let duration = start.elapsed();

    println!(
        "PERF: Processed {} levels of nested JSON in {:?}",
        DEPTH, duration
    );

    // 1. Functional check
    assert_eq!(result, Ok(expected));

    assert!(
        duration.as_millis() < 500,
        "Performance test took too long: {:?}. This may indicate a performance regression.",
        duration
    );
}
