# telomere JSON

**A lightweight, streaming JSON-capping library.**

Telomere processes an incomplete or streaming JSON object or array and provides the exact sequence of characters required to close it, making it syntactically complete.

---

### Purpose

The primary goal of `telomere-json` is to **cap an incomplete JSON stream in real time**. It was designed specifically for use cases like processing structured JSON output from Large Language Models, where the stream might be cut off before all structures are properly closed.

It is a specialized lexer and state machine, not a general-purpose JSON parser or validator. Its key function is to distinguish between a JSON stream that is merely incomplete and one that is structurally corrupted, and to provide the "cap" for the former.

### Key Features

- **Streaming-First**: Processes JSON chunk-by-chunk via `process_delta`.
- **Intelligent Completion**: Calculates the precise closing characters required (e.g., `"}`, `"]}`, `"`).
- **Robust Error Handling**: Differentiates between two key states:
  - `Error::NotClosable`: The stream is incomplete but not yet invalid (e.g., waiting for a value after a colon). More data may resolve this.
  - `Error::Corrupted`: The stream has a definitive syntax violation (e.g., `[}`) and can never be completed.
- **Lightweight**: No heavy dependencies and a focused API.

### Best Used For

`telomere` is ideal for any situation where you receive potentially incomplete JSON and need to safely make it whole.

- **Capping AI/LLM structured data streams**: If a language model is generating JSON and hits a token limit, `telomere` can correctly close the output it has produced so far.
- **Real-time data feeds**: Safely terminating partial JSON objects from a log or data feed that may be interrupted.
- **Repairing truncated files**: Providing the closing characters for a JSON file that was not fully written to disk.

### Current Weaknesses & Limitations

- **Unicode Escape Sequences**: The library's handling of Unicode escape sequences (`\uXXXX`) is currently a known weakness. While it correctly identifies that a stream containing a partial escape like `\u` is temporarily unclosable (`Error::NotClosable`), it does not validate the full 4-digit hex code or complete it. This feature may be improved in the future.
- **Not a Validator**: `telomere` is **not a JSON validator**. It does not validate data types, check for duplicate keys, or enforce all the rules of the JSON specification. Its purpose is strictly to provide the closing characters for a structurally sound but incomplete stream.

### Quick Start

```rust
// main.rs
use telomere::{JSONBalancer, Error};

fn main() {
    let mut balancer = JSONBalancer::new();
    let mut result = Ok(String::new());

    // Simulate receiving chunks of an incomplete JSON stream
    let deltas = &["{", r#""a""#, ":", "[", "1"]; // e.g., {"a":[1

    for delta in deltas {
        // The balancer might return NotClosable on intermediate chunks
        // but we only care about the final result here.
        result = balancer.process_delta(delta);
    }

    // After the last delta, we expect a successful completion
    match result {
        Ok(completion) => {
            // completion will be "]}"
            println!("JSON completed with: {}", completion);
        }
        Err(Error::NotClosable) => {
            println!("The JSON is incomplete but not corrupted.");
        }
        Err(Error::Corrupted) => {
            println!("The JSON is structurally corrupted.");
        }
        // Handle other potential errors...
        _ => {}
    }
}
```
