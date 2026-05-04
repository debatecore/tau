#!/usr/bin/env python3
import re
import subprocess
import sys
import tomllib
from pathlib import Path

SEMVER_RE = re.compile(
    r"^(0|[1-9]\d*)\."
    r"(0|[1-9]\d*)\."
    r"(0|[1-9]\d*)"
    r"(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$"
)

VERSION_FILES = ["Cargo.toml", "pyproject.toml"]

def git_show(ref: str, path: str):
    try:
        return subprocess.check_output(
            ["git", "show", f"{ref}:{path}"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
    except subprocess.CalledProcessError:
        return ""

def cargo_version(content: str):
    data = tomllib.loads(content)

    if "package" in data and "version" in data["package"]:
        return data["package"]["version"]

    if "workspace" in data:
        workspace_package = data["workspace"].get("package", {})
        if "version" in workspace_package:
            return workspace_package["version"]

    raise ValueError("Could not find version in Cargo.toml")

def pyproject_version(content: str):
    data = tomllib.loads(content)

    if "project" in data and "version" in data["project"]:
        return data["project"]["version"]

    poetry = data.get("tool", {}).get("poetry", {})
    if "version" in poetry:
        return poetry["version"]

    raise ValueError("Could not find version in pyproject.toml")

def extract_version(path: str, content: str):
    if path == "Cargo.toml":
        return cargo_version(content)

    if path == "pyproject.toml":
        return pyproject_version(content)

    raise ValueError(f"Unsupported file: {path}")

def version_core(version: str) -> tuple[int, int, int]:
    match = SEMVER_RE.match(version)
    if not match:
        raise ValueError(f"{version!r} is not valid semantic versioning")

    return tuple(int(part) for part in match.groups()[:3])

def main():
    base_ref = sys.argv[1] if len(sys.argv) > 1 else "origin/master"

    current_versions = {}
    base_versions = {}

    for path in VERSION_FILES:
        current_content = Path(path).read_text()
        base_content = git_show(base_ref, path)

        if not base_content:
            print(f"::error::{path} does not exist in base ref {base_ref}")
            return 1

        current_versions[path] = extract_version(path, current_content)
        base_versions[path] = extract_version(path, base_content)

    cargo_current = current_versions["Cargo.toml"]
    pyproject_current = current_versions["pyproject.toml"]

    if cargo_current != pyproject_current:
        print(
            "::error::Version numbers must match in Cargo.toml and pyproject.toml. "
            f"Found Cargo.toml={cargo_current}, pyproject.toml={pyproject_current}."
        )
        return 1

    if not SEMVER_RE.match(cargo_current):
        print(
            "::error::Version number must follow Semantic Versioning 2.0.0. "
            f"Found {cargo_current!r}."
        )
        return 1

    for path in VERSION_FILES:
        if current_versions[path] == base_versions[path]:
            print(
                "::error::A version number must be changed in Cargo.toml and "
                "pyproject.toml for every merged PR. Please bump the version "
                "according to Semantic Versioning 2.0.0: "
                "https://semver.org/spec/v2.0.0.html"
            )
            return 1

    cargo_base = base_versions["Cargo.toml"]

    if version_core(cargo_current) <= version_core(cargo_base):
        print(
            "::error::Version must be increased according to Semantic Versioning "
            f"2.0.0. Base version is {cargo_base}, PR version is {cargo_current}."
        )
        return 1

    print(f"Version check passed: {cargo_base} -> {cargo_current}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())