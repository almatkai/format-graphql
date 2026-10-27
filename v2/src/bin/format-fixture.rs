//! Reproduction utility; production uses the library and compiled adapter directly.
use serde_json::{json, Value};
use std::io::{self, BufRead};
use tezbar_format_graphql::{format, Indentation};
fn main() {
    for line in io::stdin().lock().lines() {
        let value: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let result = Indentation::from_value(value["indent"].as_str().unwrap_or("2"))
            .and_then(|i| format(value["input"].as_str().unwrap(), i));
        println!(
            "{}",
            match result {
                Ok(output) => json!({"output":output}),
                Err(e) => json!({"error":e.to_string()}),
            }
        );
    }
}
