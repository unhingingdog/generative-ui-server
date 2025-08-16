//! Regression tests for specific, previously-fixed bugs.

use telomere_json::{Error, JSONBalancer};

/// This test replicates a specific bug found in a real-world scenario.
/// The bug occurred when a delta containing a single closing brace `}` was
/// processed immediately after a large, deeply nested structure.
///
/// The core of the bug was that after the second-to-last object in an array
/// was closed, the parser's state became incorrect, leading to a `Corrupted`
/// error when the next delimiter was processed.
#[test]
fn regression_close_object_as_last_item_in_array() {
    let mut balancer = JSONBalancer::new();

    let initial_chunk = r#"{ "type": "container", "children": [ { "type": "heading", "level": 2, "content": "Let’s get started" }, { "type": "paragraph", "content": "Hi! Please provide your name and what you need help with." }, { "type": "form", "children": [ { "type": "input", "queryId": "user_name", "queryContent": "Your name" }, { "type": "input", "queryId": "user_need", "queryContent": "What do you need help with?" } ] "#;

    // Process the first part of the stream.
    // After this, the parser state is inside the "form" object, having just completed the "children" array.
    let _ = balancer.process_delta(initial_chunk);

    // This delta now correctly closes the "form" object.
    let failing_delta = "}";
    let result = balancer.process_delta(failing_delta);

    let expected_completion = Ok("]}".to_string());

    assert_eq!(result, expected_completion);
}
