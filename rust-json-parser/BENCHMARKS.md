# Benchmark History

Tracks parsing performance across three implementations over time.
Each run uses `uv run python -m rust_json_parser --benchmark` with `--release` builds.

## Run 1 — Baseline (2026-05-25)

No optimizations applied. Starting point.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000482s | 0.000640s | 0.000578s | 1.33x faster | 1.20x faster |
| twitter.json | 568 KB | 3.455328s | 2.355477s | 1.971979s | 1.47x slower | 1.75x slower |
| citm_catalog.json | 1.7 MB | 7.846024s | 5.752500s | 5.587131s | 1.36x slower | 1.40x slower |
| canada.json | 2.3 MB | 38.180599s | 25.742842s | 31.482822s | 1.48x slower | 1.21x slower |

### Observations

- Rust is faster than both Python implementations on verysmall.json only; slower than both on twitter, citm_catalog, and canada.

## Run 2 — `String::with_capacity(20)` in `read_digit` (2026-05-25)

Pre-allocate the number-building string in `Tokenizer::read_digit`.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000924s | 0.000499s | 0.000601s | 1.85x slower | 1.54x slower |
| twitter.json | 568 KB | 3.436280s | 2.416657s | 2.080262s | 1.42x slower | 1.65x slower |
| citm_catalog.json | 1.7 MB | 6.477067s | 5.132289s | 5.140006s | 1.26x slower | 1.26x slower |
| canada.json | 2.3 MB | 11.371001s | 17.808122s | 19.939192s | 1.57x faster | 1.75x faster |

### Observations

- canada.json: 38.18s (Run 1) → 11.37s.
- citm_catalog.json: 7.85s → 6.48s.
- twitter.json: 3.46s → 3.44s.
- verysmall.json: 0.000482s → 0.000924s.

## Run 3 — `String::with_capacity(64)` in `read_string` (2026-05-25)

Pre-allocate the string-building buffer in `Tokenizer::read_string`.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000308s | 0.000484s | 0.000595s | 1.57x faster | 1.93x faster |
| twitter.json | 568 KB | 3.577701s | 2.495969s | 2.098070s | 1.43x slower | 1.71x slower |
| citm_catalog.json | 1.7 MB | 7.277370s | 5.886443s | 6.593590s | 1.24x slower | 1.10x slower |
| canada.json | 2.3 MB | 13.249444s | 21.023933s | 22.410327s | 1.59x faster | 1.69x faster |

### Observations

- twitter.json: 3.44s (Run 2) → 3.58s.
- citm_catalog.json: 6.48s → 7.28s.
- canada.json: 11.37s → 13.25s.
- verysmall.json: 0.000924s → 0.000308s.

## Run 4 — `String::with_capacity(256)` in `read_string` (2026-05-25)

Increased the `read_string` buffer pre-allocation from 64 to 256 bytes.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000280s | 0.000489s | 0.000667s | 1.75x faster | 2.38x faster |
| twitter.json | 568 KB | 3.922787s | 2.527915s | 1.980844s | 1.55x slower | 1.98x slower |
| citm_catalog.json | 1.7 MB | 6.941263s | 5.191806s | 5.797132s | 1.34x slower | 1.20x slower |
| canada.json | 2.3 MB | 11.351446s | 18.338183s | 20.493925s | 1.62x faster | 1.81x faster |

### Observations

- twitter.json: 3.58s (Run 3) → 3.92s.
- citm_catalog.json: 7.28s → 6.94s.
- canada.json: 13.25s → 11.35s.
- verysmall.json: 0.000308s → 0.000280s.
- Profiling showed `HashMap` operations in `parse_object` at 6.1% of samples.

## Run 5 — Replace `Vec<char>` with `&str` in Tokenizer (2026-05-26)

Eliminated the `input.chars().collect()` allocation in `Tokenizer::new`. The struct
now borrows the input `&str` and uses byte-level access via `.as_bytes()`. Added a
lifetime annotation (`'a`) on the `Tokenizer` struct. Benchmark run with samply and a
browser active.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000282s | 0.000513s | 0.000709s | 1.82x faster | 2.51x faster |
| twitter.json | 568 KB | 3.252252s | 2.256309s | 1.951530s | 1.44x slower | 1.67x slower |
| citm_catalog.json | 1.7 MB | 6.169968s | 5.151962s | 6.099023s | 1.20x slower | 1.01x slower |
| canada.json | 2.3 MB | 31.270613s | 31.370237s | 25.175580s | 1.00x faster | 1.24x slower |

### Observations

- canada.json: 11.35s (Run 4) → 31.27s, run under profiler/browser load (see Run 6 for the clean re-run).
- twitter.json: 3.92s → 3.25s.
- citm_catalog.json: 6.94s → 6.17s.
- verysmall.json: 0.000280s → 0.000282s.

## Run 6 — Re-benchmark `&str` refactor, no profiler load (2026-05-26)

Same code as Run 5. Re-run with no profiler and no browser active.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000315s | 0.000498s | 0.000615s | 1.58x faster | 1.95x faster |
| twitter.json | 568 KB | 3.484934s | 2.266166s | 1.943392s | 1.54x slower | 1.79x slower |
| citm_catalog.json | 1.7 MB | 5.740152s | 4.971308s | 4.981171s | 1.15x slower | 1.15x slower |
| canada.json | 2.3 MB | 10.585388s | 17.936120s | 19.614882s | 1.69x faster | 1.85x faster |

### Observations

- canada.json: 31.27s (Run 5, under load) → 10.59s.
- twitter.json: 3.25s → 3.48s.
- citm_catalog.json: 6.17s → 5.74s.
- verysmall.json: 0.000282s → 0.000315s.

## Run 7 — `HashMap::with_capacity(16)` + `FxHashMap` in parse_object (2026-05-26)

Pre-size the HashMap to 16 entries in `parse_object`, and replace
`std::collections::HashMap` with `rustc_hash::FxHashMap` throughout (parser + JsonValue).

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000326s | 0.000494s | 0.000598s | 1.52x faster | 1.83x faster |
| twitter.json | 568 KB | 3.261620s | 2.304444s | 1.907692s | 1.42x slower | 1.71x slower |
| citm_catalog.json | 1.7 MB | 5.706554s | 4.949865s | 5.125027s | 1.15x slower | 1.11x slower |
| canada.json | 2.3 MB | 11.198368s | 18.180192s | 19.823779s | 1.62x faster | 1.77x faster |

### Observations

- canada.json: 10.59s (Run 6) → 11.20s.
- twitter.json: 3.48s → 3.26s.
- citm_catalog.json: 5.74s → 5.71s.
- verysmall.json: 0.000315s → 0.000326s.

## Run 8 — Replace index-based token cursor with `VecDeque::pop_front` (2026-05-28)

**Machine change: MacBook Pro M4 Max (Runs 1–7 were on MacBook Air M3). Numbers are not
directly comparable to Runs 1–7.**

Replaced the index-based cursor in `JsonParser` with `VecDeque::pop_front`: `front()`
peeks, `pop_front()` consumes. The `position` field (renamed from `current`) is now a
token counter for error reporting. `is_at_end()` removed in favor of `peek().is_none()`.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000208s | 0.000447s | 0.000939s | 2.15x faster | 4.51x faster |
| twitter.json | 568 KB | 1.648916s | 2.324019s | 1.743994s | 1.41x faster | 1.06x faster |
| citm_catalog.json | 1.7 MB | 5.152578s | 4.643702s | 4.178195s | 1.11x slower | 1.23x slower |
| canada.json | 2.3 MB | 7.759987s | 16.465308s | 18.469949s | 2.12x faster | 2.38x faster |

### Observations

- New M4 Max baseline; not comparable to Runs 1–7.
- twitter.json (1.65s): faster than both json(C) (2.32s) and simplejson (1.74s).
- canada.json: 7.76s; 2.12x faster than json(C).
- citm_catalog.json: 5.15s; 1.11x slower than json(C).

## Run 9 — `Cow<'a, str>` borrow fast-path in `read_string`, under profiler load (2026-05-30)

Added a borrow fast-path in `Tokenizer::read_string`. A single `find(['\\', '"'])` scan
locates the first special byte; when a closing `"` arrives before any `\`, the literal has
no escapes and the content is returned as `Cow::Borrowed`, slicing into the input with no
allocation or copy. Literals containing escapes fall through to the owned escape-processing
loop (`Cow::Owned`). Requires the `Cow`/lifetime plumbing threaded through `Token<'a>`,
`JsonValue<'a>`, and `JsonParser<'a>`. Benchmark run with samply and the Firefox Profiler
active.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000193s | 0.000511s | 0.009472s | 2.64x faster | 49.02x faster |
| twitter.json | 568 KB | 1.475477s | 2.508074s | 2.059596s | 1.70x faster | 1.40x faster |
| citm_catalog.json | 1.7 MB | 6.648539s | 5.468216s | 5.505258s | 1.22x slower | 1.21x slower |
| canada.json | 2.3 MB | 9.459157s | 21.379794s | 23.699132s | 2.26x faster | 2.51x faster |

### Observations

- Run taken with samply and the Firefox Profiler active.
- Profiling showed heap alloc + dealloc at ~14% of samples (`libsystem_malloc` 6.9% + `alloc::alloc::dealloc` 6.8%), down from the ~50% baseline.

## Run 10 — `Cow<'a, str>` borrow fast-path in `read_string`, clean re-run (2026-05-30)

Same code as Run 9. Re-run with no profiler attached; two consecutive runs agreed within ~1%.

| Fixture | Size | Rust | Python json (C) | simplejson | vs json (C) | vs simplejson |
|---|---|---|---|---|---|---|
| verysmall.json | 7 B | 0.000197s | 0.000492s | 0.000593s | 2.50x faster | 3.01x faster |
| twitter.json | 568 KB | 1.369181s | 2.401783s | 1.989226s | 1.75x faster | 1.45x faster |
| citm_catalog.json | 1.7 MB | 6.580324s | 5.221518s | 5.399443s | 1.26x slower | 1.22x slower |
| canada.json | 2.3 MB | 9.296854s | 20.415003s | 22.898312s | 2.20x faster | 2.46x faster |

### Observations

- twitter.json: 1.65s (Run 8) → 1.37s.
- citm_catalog.json: 5.15s (Run 8) → 6.58s.
- canada.json: 7.76s (Run 8) → 9.30s.
- 78 unit tests + 13 doctests pass under `--no-default-features`.
