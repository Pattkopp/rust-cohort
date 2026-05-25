# Benchmark History

Tracks parsing performance across three implementations over time.
Each run uses `uv run python -m rust_json_parser --benchmark` with `--release` builds.

## Run 1 — Baseline (2026-05-25)

No optimizations applied yet. This is the starting point.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000482s | 0.000640s | 0.000578s | 1.33x faster | 1.20x faster |
| twitter.json | 568 KB | 3.455328s | 2.355477s | 1.971979s | 1.47x slower | 1.75x slower |
| citm_catalog.json | 1.7 MB | 7.846024s | 5.752500s | 5.587131s | 1.36x slower | 1.40x slower |
| canada.json | 2.3 MB | 38.180599s | 25.742842s | 31.482822s | 1.48x slower | 1.21x slower |

### Observations

- Rust wins on tiny input (7 bytes) but loses on all larger files.
- The gap widens with input size, suggesting per-token overhead.
- Slower than even pure Python (simplejson) — indicates a systemic issue, not just algorithm choice.

## Run 2 — `String::with_capacity(20)` in `read_digit` (2026-05-25)

Pre-allocate the number-building string in `Tokenizer::read_digit` to avoid
repeated heap reallocations during number parsing.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000924s | 0.000499s | 0.000601s | 1.85x slower | 1.54x slower |
| twitter.json | 568 KB | 3.436280s | 2.416657s | 2.080262s | 1.42x slower | 1.65x slower |
| citm_catalog.json | 1.7 MB | 6.477067s | 5.132289s | 5.140006s | 1.26x slower | 1.26x slower |
| canada.json | 2.3 MB | 11.371001s | 17.808122s | 19.939192s | 1.57x faster | 1.75x faster |

### Observations

- canada.json improved dramatically: from 38.2s → 11.4s (3.4x speedup), now faster than both Python implementations.
- citm_catalog.json improved from 7.8s → 6.5s (17% faster), gap with Python narrowed.
- twitter.json roughly unchanged — string-heavy file, so number allocation was not the bottleneck there.
- verysmall.json regressed slightly, likely measurement noise on a 7-byte input.
