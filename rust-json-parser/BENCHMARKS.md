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

## Run 5 — Replace `Vec<char>` with `&str` in Tokenizer (2026-05-26)

Eliminated the `input.chars().collect()` allocation in `Tokenizer::new`. The struct
now borrows the input `&str` directly and uses byte-level access via `.as_bytes()`.
Introduced lifetime annotations (`'a`) on the `Tokenizer` struct.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000282s | 0.000513s | 0.000709s | 1.82x faster | 2.51x faster |
| twitter.json | 568 KB | 3.252252s | 2.256309s | 1.951530s | 1.44x slower | 1.67x slower |
| citm_catalog.json | 1.7 MB | 6.169968s | 5.151962s | 6.099023s | 1.20x slower | 1.01x slower |
| canada.json | 2.3 MB | 31.270613s | 31.370237s | 25.175580s | 1.00x faster | 1.24x slower |

### Observations

- canada.json regressed significantly: 11.4s (Run 4) → 31.3s. The `Vec<char>` elimination should have helped, but byte-level access with `u8`-to-`char` casting in `advance()` may be introducing overhead on this number-heavy file (2.2M of coordinate data parsed character by character).
- twitter.json improved slightly: 3.92s (Run 4) → 3.25s (7% better than Run 3's 3.58s).
- citm_catalog.json improved: 6.94s (Run 4) → 6.17s, now nearly matching simplejson.
- The canada.json regression needs profiling. Likely cause: the byte-to-char conversion in `advance()` runs millions of times for number parsing, and the `.map()` closure may not be optimizing as well as the old `.copied()` on `Vec<char>`.

## Run 6 — Re-benchmark `&str` refactor under lower system load (2026-05-26)

Same code as Run 5. Re-run to verify whether the canada.json regression was real
or caused by system load during Run 5 (profiler and browser were active).

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000315s | 0.000498s | 0.000615s | 1.58x faster | 1.95x faster |
| twitter.json | 568 KB | 3.484934s | 2.266166s | 1.943392s | 1.54x slower | 1.79x slower |
| citm_catalog.json | 1.7 MB | 5.740152s | 4.971308s | 4.981171s | 1.15x slower | 1.15x slower |
| canada.json | 2.3 MB | 10.585388s | 17.936120s | 19.614882s | 1.69x faster | 1.85x faster |

### Observations

- canada.json regression was NOT real: 10.6s here vs 31.3s in Run 5. The Run 5 measurement was inflated by system load (profiler + browser). True performance is consistent with Run 2 (11.4s) and slightly better.
- twitter.json at 3.48s — slightly above Run 5's 3.25s, likely run-to-run variance. Still improved vs Run 4 (3.92s).
- citm_catalog.json at 5.74s — improved from Run 5's 6.17s and Run 4's 6.94s.
- The `&str` refactor is a net positive across all files. Next target: HashMap overhead in `parse_object` (~17% of canada.json profile).

## Run 7 — `HashMap::with_capacity(16)` + `FxHashMap` in parse_object (2026-05-26)

Two changes: pre-size HashMap to 16 entries in `parse_object`, and replace
`std::collections::HashMap` with `rustc_hash::FxHashMap` throughout (parser + JsonValue).
FxHash is a faster, non-cryptographic hasher suited for trusted input like JSON keys.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000326s | 0.000494s | 0.000598s | 1.52x faster | 1.83x faster |
| twitter.json | 568 KB | 3.261620s | 2.304444s | 1.907692s | 1.42x slower | 1.71x slower |
| citm_catalog.json | 1.7 MB | 5.706554s | 4.949865s | 5.125027s | 1.15x slower | 1.11x slower |
| canada.json | 2.3 MB | 11.198368s | 18.180192s | 19.823779s | 1.62x faster | 1.77x faster |

### Observations

- canada.json at 11.2s — consistent with Run 6 (10.6s). The FxHashMap swap and pre-sizing did not produce a measurable improvement here, likely because canada.json has very few object keys (mostly arrays of number coordinates).
- twitter.json at 3.26s — matching Run 5 (3.25s) and Run 6 (3.48s). No measurable change from FxHashMap, though this file has many small objects.
- citm_catalog.json at 5.71s — consistent with Run 6 (5.74s). Stable.
- The FxHashMap change is architecturally correct (removes unnecessary cryptographic hashing overhead) but the benchmark files don't stress object-key hashing enough to show a clear difference. The improvement may be more visible on JSON with larger objects or more keys. Next target: Token cloning in parser's `advance()` (~1% overhead).

## Run 8 — Replace index-based token cursor with `VecDeque::pop_front` (2026-05-28)

**Machine change: MacBook Pro M4 Max (Runs 1–7 were on MacBook Air M3).
Numbers are not directly comparable to previous runs due to the hardware change.**

Replaced the index-based cursor in `JsonParser` with `VecDeque::pop_front`. The parser
no longer maintains an index into the token buffer — `front()` peeks, `pop_front()`
consumes. The `position` field (renamed from `current`) is now purely a token counter
for error reporting. `is_at_end()` was eliminated in favor of `peek().is_none()`.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000208s | 0.000447s | 0.000939s | 2.15x faster | 4.51x faster |
| twitter.json | 568 KB | 1.648916s | 2.324019s | 1.743994s | 1.41x faster | 1.06x faster |
| citm_catalog.json | 1.7 MB | 5.152578s | 4.643702s | 4.178195s | 1.11x slower | 1.23x slower |
| canada.json | 2.3 MB | 7.759987s | 16.465308s | 18.469949s | 2.12x faster | 2.38x faster |

### Observations

- **twitter.json crossed the line**: Rust now faster than both Python implementations. Previous best on M3 was 3.25s (1.44x slower than json C); now 1.65s and 1.41x faster. The `pop_front` change eliminates redundant indexing on every token access, which compounds over twitter.json's ~30k tokens.
- canada.json at 7.76s, down from 11.2s (Run 7). Rust is now 2.12x faster than Python's json(C).
- citm_catalog.json remains the one file where Rust trails — 1.11x slower than json(C). This file is heavy on string keys and object structure; the remaining overhead is likely in string allocation and HashMap insertion.
- verysmall.json shows the M4 Max baseline: 4.51x faster than simplejson, up from 1.83x on M3.
- Hardware change accounts for a significant portion of the raw time improvement. The architectural change (pop_front vs index) contributes the twitter.json ratio flip — that comparison is against Python running on the same M4 Max hardware.
