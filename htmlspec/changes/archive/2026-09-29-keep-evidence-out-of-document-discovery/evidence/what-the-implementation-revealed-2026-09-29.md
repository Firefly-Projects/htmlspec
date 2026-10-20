# What the implementation revealed

**Date:** 2026-09-29
**Task:** 5.4

## 1. The earlier record understated the damage `format` does

The 2026-09-09 record says `format` "rewrote one of them". It had run `format` over one directory.
Run over a whole project on HEAD, `format` rewrote **all seven** HTML records this repository holds.
See `evidence-read-as-documents-2026-09-29.md`. The difference is scope, not behavior: every record
names a stylesheet revision that has since been replaced, and `format` repoints each one at the
current store.

## 2. The asset check reads bytes, not paths, so a record can look conforming by accident

The first test record was the specification template placed in a change's `evidence/`, with its
stylesheet addresses written for a document at the store's own level. From three directories
deeper, those paths lead nowhere. I expected `asset/unresolved`, and `validate` reported **0
errors**.

The reason: `asset/unresolved` asks whether the project stores bytes at the named *address*, and
the digest in the address was current. A wrong relative path to bytes the project does hold is not
an error. It is something `format` repairs. So a record written recently enough validates as a
document, and still gets rewritten by `format`.

The tests therefore use the two stylesheet addresses this repository's own records carry,
`e4f5de7…` and `0350261…`, which a freshly established store does not hold. That is the realistic
case in any event. A record is old, and its stylesheet has moved.

## 3. Outside a project a missing stylesheet is not a violation

With no project there is no store to be missing bytes from, so a record with superseded addresses
validates clean outside a project. The "outside a project every file is a document" test first
failed on HEAD for this reason. That was a fault in the test, not in the behavior. The record now
also carries a dangling unit reference (`refs/unresolved`), which fails wherever the file sits, so
"walked" and "checked and clean" cannot be confused.

## 4. Only promotion can tell where discovery resolves its project from

Mutation 4 in `mutation-testing-2026-09-29.md` resolves the project from the working directory
instead of from the walk's root. No `validate` or `format` test catches it, because each one that
expects an exclusion runs from inside the project it walks. Promotion catches it, because promotion
runs from above the staged tree. That makes the design's choice to resolve from the root
observable in exactly one command, and it is the command this change exists for. A contributor who
runs `htmlspec validate other-project/htmlspec` from elsewhere would meet the same distinction,
with no test pinning it through `validate`.

## 5. Two positional questions now share one canonicalization rule

`topology.rs` asked whether a store is the project's by canonical path prefix, with a private
`canonical` helper. This change asks whether a directory is a change's evidence by canonical parent,
and needed the same rule. The helper moved to `project.rs` as `pub(crate) fn canonical`, beside
the layout both questions compare against. The asset topology tests pass unchanged. Keeping two
copies would have let the two questions drift in how they treat a path that cannot be resolved.

## 6. Until this change, the contract did not say where an html-spec change keeps its evidence

Only the non-normative `migrate` instructions named `evidence/**`. The OpenSpec survey in
`migrate.rs` classifies `evidence/` for a source project, and `ProjectRoot` had no evidence location
at all. So the new artifact-kinds requirement states two things: where evidence lives, and that it
is not a document. The first had to exist before the second could be said.

## 7. The release runbook's second hazard is unchanged, and it bit here too

Regenerating the installed instructions in task 4.1 ran `htmlspec init --tools claude,codex,pi`,
which created `htmlspec/` with eight assets and a `spec.nix` at `0.0.0`. Both were removed by hand.
That step disappears only when this repository becomes an html-spec project, which is the
migration this change unblocks.

## 8. A review found the two new requirements contradicting each other

The first version said, in artifact-kinds, that no operation rewrites a file beneath a change's
evidence. In toolchain-packaging it said that a path the contributor names is operated on as named.
For `format` those cannot both hold, and the code followed the second. Run against the scratch
project's pristine copy, `htmlspec format htmlspec/changes/measure-evidence/evidence` exited 0 and
rewrote all seven records.

Nothing before the review caught it. The tests for a named path used `validate`, the only command
where "operated on as named" and "never rewritten" agree. The mutation pass could not find it either,
because it mutated the code toward the design, and the design had the gap. The fix separates
checking from writing. A named record is checked as named, and `format`'s writing mode refuses it
before writing anything.
