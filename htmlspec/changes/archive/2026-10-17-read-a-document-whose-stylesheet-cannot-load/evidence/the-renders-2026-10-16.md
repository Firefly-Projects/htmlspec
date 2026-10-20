# The fallback, rendered with and without its store, 2026-10-16

Two copies of this repository's `htmlspec/specs/runbook-portability/spec.html` were laid out in a
scratch directory beside a copy of `htmlspec/assets/`, at the same depth as in the repository:

- `htmlspec/specs/with/spec.html`, given the fallback by `htmlspec format --fallback`, built from
  this change at `7323b0a`. It reported `gains the fallback stylesheet`, then `formatted`.
- `htmlspec/specs/without/spec.html`, the specification as it stands, without the fallback.

Each was rendered by Chrome 154 headless, with `--window-size=1280,1600`, a virtual time budget of
8000 ms, and `--screenshot`. Then the store was renamed out of reach and each was rendered again.

| Render | Store | Fallback | SHA-256 of the screenshot, first 16 hex |
| --- | --- | --- | --- |
| A | reachable | carried | `abb92f5d652ecbec` |
| B | reachable | absent | `abb92f5d652ecbec` |
| C | unreachable | carried | `1c9d93133041882a` |
| D | unreachable | absent | `71f7c79254cb6f14` |

**A and B are identical byte for byte.** With its stylesheet loaded, the document with the fallback
renders exactly as the document without it. That is the inertness the static test in
`crates/htmlspec-assets/tests/fallback.rs` predicts, observed.

**C is readable and D is not.** Without the store, C rendered as a centred column on the house
paper, in the house serif and ink, with the masthead's code and the migrated-from path in a
monospace face, and the skip link hidden. D rendered edge to edge on white in the browser's default
serif, with the skip link showing at the top left. D is what Safari showed the user for documents
opened from disk, and C is what such a reader now gets from a document carrying the fallback.

C is not the design. It has no contents column, the masthead's fields stack as a plain definition
list, each requirement's identifier runs into its heading, and scenarios have no styling. The
fallback restores a readable page and nothing more, as the design's risk "Without its stylesheet, a
page is readable but not designed" accepts.

Safari itself was not driven: it cannot be run headless here. The screenshots were kept in the
session's scratch directory, not in the repository.
