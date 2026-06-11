#!/usr/bin/env python3
from pathlib import Path

from check_version import extract_version

VERSION_FILE = "Cargo.toml"


def main():
    print(extract_version(VERSION_FILE, Path(VERSION_FILE).read_text()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
