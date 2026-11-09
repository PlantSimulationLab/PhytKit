"""`pk` from the Python package: the same command line, output and exit codes as the `pk` binary."""
import sys

from ._phytkit import main as _main


def main() -> int:
    text, code = _main(sys.argv[1:])
    sys.stdout.write(text)
    return code


if __name__ == "__main__":
    sys.exit(main())
