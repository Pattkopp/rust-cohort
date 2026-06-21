# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A JSON parser written from scratch in Rust (no `serde_json`), exposed both as a Rust library and as a Python extension module via PyO3. The crate lives in `rust-json-parser/`; the repository root holds only that crate plus benchmark/learning notes. This is a structured learning project: work is organized into branches `week1`–`week6` (merged into `main`), and the parser is incrementally optimized with each change measured and recorded in `BENCHMARKS.md`.

All commands below assume the working directory is `rust-json-parser/`.

## Build and Test Commands

The crate has a default feature `python` that enables `pyo3` with `extension-module`. That feature tells PyO3 **not** to link libpython, which is correct when maturin produces a `.so`, but makes any `cargo` command that links a runnable artifact (the test harness, the `rust-json-parser` binary, examples) fail with "symbol(s) not found" undefined-Python-symbol link errors.

Therefore:

- **Rust tests:** `cargo test --no-default-features` (unit tests + doctests). This is the canonical Rust test command; `cargo test` alone fails to link.
- **Run a single Rust test:** `cargo test --no-default-features <test_name_substring>`.
- **Run the Rust CLI demo:** `cargo run --no-default-features`.
- **Lint / format / check:** `cargo fmt`, `cargo clippy --all-targets --no-default-features -- -D warnings`, `cargo check --no-default-features`.
- **Build the Python extension:** `uv run maturin develop` (debug) or `uv run maturin develop --release` (required before benchmarking). This compiles the crate *with* the `python` feature and installs `_rust_json_parser` into the venv. Building the extension is maturin's job, not `cargo`'s.
- **Python integration tests:** `uv run pytest` (in `tests/test_python_integration.py`; requires a prior `maturin develop`).
- **CLI parse:** `uv run python -m rust_json_parser '<json>'`, a file path, or piped stdin; prints pretty-printed JSON.
- **Benchmark:** `uv run python -m rust_json_parser --benchmark` (compares the Rust parser against Python's `json` C module and `simplejson` over the fixtures in `python/rust_json_parser/bench_data/`; needs a `--release` extension build for meaningful numbers).
- **CPU profiling:** `cargo build --example profile_bench --no-default-features --profile profiling` then `samply record target/profiling/examples/profile_bench`.

Package management: `cargo add`/`cargo remove` and `uv add`/`uv remove` only; do not hand-edit `Cargo.toml` `[dependencies]` or `pyproject.toml` dependencies. Edition 2024.

## Architecture

The pipeline is a single streaming pass: `Tokenizer` → `JsonParser` → `JsonValue`. No intermediate token collection is built.

- **`src/tokenizer.rs`** — `Tokenizer<'a>` borrows the input `&'a str` and implements `Iterator<Item = Result<Token<'a>, JsonError>>`, yielding one token at a time via byte-level (`u8`) scanning over `input.as_bytes()`. It handles whitespace, structural punctuation, numbers (parsed to `f64`), keywords (`true`/`false`/`null`), and string escapes including `\uXXXX`. `read_string` has a zero-copy fast path: a single `find(['\\', '"'])` scan; if the closing quote arrives before any backslash the literal is returned as `Cow::Borrowed` slicing directly into the input, otherwise it falls through to an owned escape-processing loop that builds a `String`.
- **`src/parser.rs`** — `JsonParser<'a>` wraps the tokenizer in `Peekable` and does recursive descent (`parse_value` → `parse_array` / `parse_object`). `JsonParser::new(input).parse()` consumes the parser and produces one value; construct a fresh parser per input. `position` is a token counter used only for error reporting.
- **`src/value.rs`** — `JsonValue<'a>` is the output tree: `Null`, `Boolean(bool)`, `Number(f64)`, `String(Cow<'a, str>)`, `Array(Vec<…>)`, `Object(FxHashMap<Cow<'a, str>, …>)`. Accessors (`is_null`, `as_str`, `as_f64`, `as_bool`, `as_array`, `as_object`, `get`), plus `Display` (compact) and `pretty_print(indent)`.
- **`src/error.rs`** — `JsonError` enum, implements `Display` + `std::error::Error`. Every variant carries a `position`, but the **unit differs by stage**: tokenizer-origin errors (`InvalidNumber`, `InvalidEscape`, `InvalidUnicode`) report a **byte offset**; parser-origin structural errors (`UnexpectedToken`, `UnexpectedEndOfInput`) report a **token index**. Preserve this distinction when touching error sites.
- **`src/python_bindings.rs`** — gated behind `#[cfg(feature = "python")]`. Defines the `_rust_json_parser` PyO3 module exposing `parse_json`, `parse_json_file`, `dumps`, and `benchmark_performance`; converts `JsonValue` → Python objects via `IntoPyObject` and `JsonError` → `PyValueError`. `py_to_json_value` checks `bool` before numeric types (a Python `bool` is an `int`), which must not be reordered.
- **`src/lib.rs`** — crate root and public re-exports; `#![warn(missing_docs)]` is on, so public items need doc comments.
- **`python/rust_json_parser/`** — the Python package: `__init__.py` re-exports the compiled symbols, `__main__.py` is the CLI/benchmark entry point. `python-source = "python"` and `module-name = "rust_json_parser._rust_json_parser"` are set in `pyproject.toml`.

### Lifetimes and zero-copy

The `'a` lifetime threads from the input `&str` through `Token<'a>`, `JsonValue<'a>`, and `JsonParser<'a>`. A parsed `JsonValue` borrows from the source string: escape-free strings and object keys are `Cow::Borrowed` slices (no allocation); only literals containing escapes are `Cow::Owned`. A `JsonValue` cannot outlive the input it was parsed from. Preserve borrowing on the no-escape paths when modifying string or key handling — it is the dominant performance property and is the subject of most of `BENCHMARKS.md`.

### Documented limitations (intentional)

Stated in the `src/lib.rs` crate docs; do not treat as bugs unless asked to change them: surrogate-pair `\u` escapes (non-BMP characters such as emoji) are rejected as `InvalidUnicode`; only the first complete top-level value is validated and any trailing content is ignored; object key order is not preserved (`FxHashMap`).

## Conventions

- Conventional Commits (`feat`, `fix`, `refactor`, `perf`, `docs`, `style`, `chore`, `test`).
- Any performance-affecting change should be benchmarked and appended as a new dated run in `BENCHMARKS.md`, following the existing per-run format (description, fixture table, observations). Note the machine in the entry; results across different machines are not comparable.
- No `unwrap()`/`expect()`/`panic!` in library code paths outside tests (a panic in a library called from Python would abort the host process). `main.rs` and the `profile_bench` example are the exceptions.
