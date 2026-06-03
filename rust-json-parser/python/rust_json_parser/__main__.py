import os.path
import sys

from rust_json_parser import benchmark_performance, dumps, parse_json, parse_json_file


def load_benchmark_fixtures() -> list[tuple[str, str]]:
    bench_dir = os.path.join(os.path.dirname(__file__), "bench_data")
    fixture_names = [
        "verysmall.json",
        "twitter.json",
        "citm_catalog.json",
        "canada.json",
    ]
    fixtures = []
    for name in fixture_names:
        path = os.path.join(bench_dir, name)
        with open(path) as f:
            content = f.read()
            label = f"{name} ({len(content)} bytes)"
            fixtures.append((label, content))
    return fixtures


def run_benchmark() -> None:
    ITERATIONS = 1000
    fixtures = load_benchmark_fixtures()
    for label, content in fixtures:
        (rust_time, python_json_time, simplejson_time) = benchmark_performance(
            content, ITERATIONS
        )

        # no guard because of std::time::Instant making 0 essentiall impossible
        json_ratio = python_json_time / rust_time
        simplejson_ratio = simplejson_time / rust_time
        print(f"{label}:")
        print(f"  {'Rust:':<18}{rust_time:.6f}s")
        print(
            f"  {'Python json (C):':<18}{python_json_time:.6f}s  (Rust is {json_ratio:.2f}x faster)"
            if json_ratio > 1
            else f"  {'Python json (C):':<18}{python_json_time:.6f}s  (Rust is {1 / json_ratio:.2f}x slower)"
        )
        print(
            f"  {'simplejson:':<18}{simplejson_time:.6f}s  (Rust is {simplejson_ratio:.2f}x faster)"
            if simplejson_ratio > 1
            else f"  {'simplejson:':<18}{simplejson_time:.6f}s  (Rust is {1 / simplejson_ratio:.2f}x slower)"
        )


def main() -> None:
    if "--benchmark" in sys.argv:
        run_benchmark()
        return
    try:
        if len(sys.argv) > 1:
            arg = sys.argv[1]
            if os.path.exists(arg):
                result = parse_json_file(arg)
            else:
                result = parse_json(arg)
        else:
            result = parse_json(sys.stdin.read())
        print(dumps(result, indent=2))
    except (ValueError, IOError) as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
