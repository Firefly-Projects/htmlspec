# The obligation reads correctly where the claims are about code

**Date:** 2026-09-06
**Task:** 5.5
**Transcript:** `transcripts/apply-on-a-real-change.md`

Task 5.2 exercised a mapping table, where every row asserts a destination path. Task 5.5 asks the
different question: does the obligation read correctly where the assertions are about code and
about the repository's own state rather than about layout?

`apply` on this change, working task 6.2 — advance the workspace version and relock offline. Real
work in the real repository, not a scratch case. The subject was chosen because it needed doing
anyway and because it is unambiguously about code and repository state.

**The prompt did not mention the contract rule.** The agent was told to read
`.claude/commands/htmlspec/apply.md` and follow it. Whether the obligation arrived at all was the
thing under test, so naming it would have decided the result.

## What it did

Advanced `0.9.0` to `0.10.0` in `Cargo.toml`, relocked with `cargo update --workspace --offline`
— 7 workspace members moved, no third-party movement, 14 lines in `Cargo.lock`.

## The obligation arrived, and it landed on the right sentence

The increment size is the claim that mattered, and the agent treated it as a claim rather than a
choice to make quietly:

> `project-bootstrap`'s requirement *Archival requires the project version to advance* states that
> "deciding what a version increment signifies, and by how much, is judgment" … So the contract
> requires the version to advance and orders the result; it does not state minor versus patch.
> **That choice is mine, and I am marking it as mine.**

That is the asymmetry `contract-authority` exists for. It also closed with an explicit
requirement-or-silence list for three separate claims.

**Every citation verified.** The requirement exists at `project-bootstrap/spec.md:49`. Both
quoted phrases are verbatim from that requirement's body — I first read one as a mismatch because
my `grep` was case-sensitive against a sentence-initial capital, and it is exact. The named
scenario *Generated instructions name their version* exists at line 196.

The one place it named something other than a requirement, it said so: the `VERSION` file
obligation is attributed to `CHANGELOG.md`'s preamble and `scripts/check-release`, with a note
that `project-bootstrap` attributes it to `toolchain-packaging` — rather than asserting a
requirement it had not read.

## The 5.4 counter-test holds here too

Facts about runs carry no citations. "`scripts/check-installed` … went red", the diff shapes, the
member counts, and "10 of 11 stages green" are all stated bare. Nothing in the transcript cites a
requirement for the agent's own reasoning about ordering or scope.

## What the run found that the task did not ask about

**The version bump restaled all 40 installed instruction files**, because `generatedBy` is written
from `CARGO_PKG_VERSION` at run time. `scripts/check-installed` — the stage added two PRs ago
after the review of PR #21 — went red and caught it.

That is the new check firing on its first release, on a case nobody predicted. Reproduced
independently: reverting the installed files while `Cargo.toml` sits at `0.10.0` turns the stage
red on all 40, and regenerating clears it.

The agent regenerated through `init --tools claude,codex,pi` rather than editing the stamp, and
said why: a hand-edited stamp is exactly the failure task 5.1 found in `cf11132`. The resulting
diff is 40 files, stamp line only — verified with
`git diff .claude .codex .pi | grep -v generatedBy`, which returns nothing.

**A caution this creates for section 6.** Any further version change before archival restales
these files again, so the regeneration is the last step of the release rather than an early one.

## One error of mine while verifying

I ran `git checkout -- .claude .codex .pi` to reproduce the red stage, which discarded the
agent's regeneration, then regenerated with a stale `0.9.0` release binary and got 40 failures I
briefly read as the agent's fault. Rebuilding at `0.10.0` and regenerating restored the correct
state. The check was right at every step; my binary was stale.

## Where the release stands

`scripts/check` is green on 10 of 11 stages. `release` fails on the absent `## 0.10.0` changelog
heading, which is task 6.3's work and is the expected state between these two tasks.
