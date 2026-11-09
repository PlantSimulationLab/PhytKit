"""`pk` from the Python package: the same command line, output and exit codes as the `pk` binary."""
import sys

from ._phytkit import main as _main


def main() -> int:
    text, code = _main(sys.argv[1:])
    # Write UTF-8 bytes, like the binary. sys.stdout.write would encode with the console's
    # code page on Windows (cp1252), which can't encode the Greek letters and dashes in
    # `pk describe` and --help, and would also turn \n into \r\n.
    sys.stdout.flush()
    sys.stdout.buffer.write(text.encode("utf-8"))
    sys.stdout.buffer.flush()
    return code


if __name__ == "__main__":
    sys.exit(main())
