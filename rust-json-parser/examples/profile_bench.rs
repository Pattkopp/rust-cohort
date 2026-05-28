//! Profiling harness for CPU profiling with [`samply`](https://github.com/mstange/samply).
//!
//! Parses `canada.json` (2.3 MB, number-heavy) in a tight loop so that
//! a sampling profiler can collect enough data to produce a useful
//! profile.
//!
//! # Usage
//!
//! Build with the dedicated `profiling` Cargo profile (optimized + debug symbols):
//!
//! ```sh
//! cargo build --example profile_bench --no-default-features --profile profiling
//! ```
//!
//! Then record and view interactively (opens Firefox Profiler in Chrome):
//!
//! ```sh
//! samply record target/profiling/examples/profile_bench
//! ```

use rust_json_parser::JsonParser;

const ITERATIONS: usize = 1000;

fn main() {
    let input = std::fs::read_to_string("python/rust_json_parser/bench_data/citm_catalog.json")
        .expect("Failed to read benchmark fixture");
    for _ in 0..ITERATIONS {
        let _result = JsonParser::new()
            .parse(&input)
            .expect("Failed to parse JSON");
    }
}
