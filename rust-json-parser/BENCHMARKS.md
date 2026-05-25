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

## Run 3 — `String::with_capacity(64)` in `read_string` (2026-05-25)

Pre-allocate the string-building buffer in `Tokenizer::read_string` to avoid
repeated heap reallocations during string parsing.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000308s | 0.000484s | 0.000595s | 1.57x faster | 1.93x faster |
| twitter.json | 568 KB | 3.577701s | 2.495969s | 2.098070s | 1.43x slower | 1.71x slower |
| citm_catalog.json | 1.7 MB | 7.277370s | 5.886443s | 6.593590s | 1.24x slower | 1.10x slower |
| canada.json | 2.3 MB | 13.249444s | 21.023933s | 22.410327s | 1.59x faster | 1.69x faster |

### Observations

- twitter.json and citm_catalog.json essentially unchanged — the `read_string` pre-allocation did not produce a measurable improvement on string-heavy files.
- canada.json slightly slower than Run 2 (13.2s vs 11.4s), likely run-to-run variance rather than a regression.
- verysmall.json recovered to faster-than-Python, confirming Run 2's regression was measurement noise.
- The string reallocation cost is not the primary bottleneck for these files. The next optimization target should address a different area — likely the `Vec<char>` conversion on line 45 or the token cloning in the parser.

## Run 4 — `String::with_capacity(256)` in `read_string` (2026-05-25)

Increased string buffer pre-allocation from 64 to 256 bytes to test whether
larger capacity reduces reallocations for longer strings (tweets, URLs, bios).

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000280s | 0.000489s | 0.000667s | 1.75x faster | 2.38x faster |
| twitter.json | 568 KB | 3.922787s | 2.527915s | 1.980844s | 1.55x slower | 1.98x slower |
| citm_catalog.json | 1.7 MB | 6.941263s | 5.191806s | 5.797132s | 1.34x slower | 1.20x slower |
| canada.json | 2.3 MB | 11.351446s | 18.338183s | 20.493925s | 1.62x faster | 1.81x faster |

### Observations

- twitter.json regressed from 3.58s (Run 3) to 3.92s — the larger allocation wastes memory on short strings (keys like "id", "name"), and the extra memory pressure offsets any reallocation savings.
- Profiling confirmed that `String::push`/`reserve` chain disappeared, but `HashMap` operations in `parse_object` became the new dominant cost (6.1%).
- canada.json stable at 11.4s, consistent with Run 2.
- Conclusion: 256 is too large. Revert to 64 and pursue a different optimization next.
