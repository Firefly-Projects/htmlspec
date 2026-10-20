"""The facts a release states in more than one script, each stated once.

`pack-release`, `render-formula`, `publish-crates`, `check-release`, and `check-publish-crates`
import this file from beside them. Each used to carry its own copy of the targets, the crates, the
User-Agent, or the reader of `[workspace.package]`, and a copy that drifted from the others would
have been found at a release, the one moment nobody can take back.

The release workflow reads two of them:

- Its build matrix is `build_matrix`'s, printed by `python3 scripts/release_facts.py matrix`, so
  the targets it builds are these, and the runner each builds on is stated beside them here. A
  matrix written out in the workflow was a second copy of the targets.
- Its `CARGO_HTTP_USER_AGENT` is a copy, compared with `USER_AGENT` by its verify job, through
  `python3 scripts/release_facts.py user-agent`.

`scripts/set-version` keeps its own reading of `[workspace.package]`, because it rewrites the
version line in place and must find exactly one; this reader only reads.

Standard library only, as every script here is: the suite runs these inside the Nix sandbox, and
the release runs them on a stock runner.

Usage:
  python3 scripts/release_facts.py user-agent                 print USER_AGENT
  python3 scripts/release_facts.py targets [--without-macos]  print the targets, one per line
  python3 scripts/release_facts.py matrix [--without-macos]   print the build matrix, as JSON
"""

from __future__ import annotations

import json
import re
import sys

# The four targets a release builds, in the order SHA256SUMS lists them.
TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
)

# The GitHub-hosted runner each target builds on: the macOS targets natively on Apple silicon and
# on Intel, and the static musl targets on Linux of the same architecture.
RUNNERS = {
    "aarch64-apple-darwin": "macos-15",
    "x86_64-apple-darwin": "macos-15-intel",
    "x86_64-unknown-linux-musl": "ubuntu-24.04",
    "aarch64-unknown-linux-musl": "ubuntu-24.04-arm",
}

# The targets that build on macOS. In a private repository a macOS minute costs as much as ten
# Linux minutes, so the release workflow builds these, and checks the formula, only in the public
# repository or on a pull request labelled `release-check`.
MACOS_TARGETS = tuple(target for target in TARGETS if target.endswith("-apple-darwin"))

# The crates the workspace publishes, in the order each one's dependencies are published before
# it. `conformance` declares `publish = false` and is not among them. `publish-crates` refuses a
# workspace that publishes any other set, and `check-publish-crates` checks this order against
# every crate's manifest.
CRATES = (
    "htmlspec-schema",
    "htmlspec-parse",
    "htmlspec-assets",
    "htmlspec-format",
    "htmlspec-validate",
    "htmlspec-cli",
)

# The public repository, the only one crates are published from.
PUBLIC_REPOSITORY = "https://github.com/Firefly-Projects/htmlspec"

# The one User-Agent every request a release makes carries: the project and its public
# repository, and nothing that names a person or a machine.
USER_AGENT = f"htmlspec-release ({PUBLIC_REPOSITORY})"


def section_fields(manifest: str, section: str) -> dict[str, str]:
    """Each `key = "value"` line under `[section]` of a Cargo manifest, by key.

    Read with a regex rather than a TOML parser: `tomllib` is standard library only from Python
    3.11, and neither the interpreter the sandbox pins nor a runner's is guaranteed to be that
    new. One quoted string per line is the shape every field read here has. A field written
    another way, as a table or across lines, is not returned, so a caller that needs it says so.
    """
    fields: dict[str, str] = {}
    current = None
    for line in manifest.splitlines():
        header = re.fullmatch(r"\s*\[([^\]]+)\]\s*", line)
        if header:
            current = header.group(1).strip()
            continue
        if current != section:
            continue
        field = re.fullmatch(r'\s*([A-Za-z0-9_-]+)\s*=\s*"([^"]*)"\s*(#.*)?', line)
        if field:
            fields[field.group(1)] = field.group(2)
    return fields


def build_targets(macos: bool) -> tuple[str, ...]:
    """The targets a run builds, in TARGETS' order: all four, or all but the macOS ones."""
    return tuple(target for target in TARGETS if macos or target not in MACOS_TARGETS)


def build_matrix(macos: bool) -> dict[str, list[dict[str, str]]]:
    """The release workflow's build matrix: one entry per target a run builds, with its runner."""
    return {"include": [{"target": target, "runner": RUNNERS[target]} for target in build_targets(macos)]}


def main(argv: list[str]) -> int:
    if argv == ["user-agent"]:
        print(USER_AGENT)
        return 0
    if argv[:1] in (["targets"], ["matrix"]) and argv[1:] in ([], ["--without-macos"]):
        macos = not argv[1:]
        if argv[0] == "targets":
            print("\n".join(build_targets(macos)))
        else:
            print(json.dumps(build_matrix(macos), separators=(",", ":")))
        return 0
    print("usage: release_facts.py {user-agent | targets [--without-macos] | matrix [--without-macos]}",
          file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
