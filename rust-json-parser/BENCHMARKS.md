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
