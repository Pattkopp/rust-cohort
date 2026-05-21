import os.path
import sys

from rust_json_parser import dumps, parse_json, parse_json_file


def main():
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
