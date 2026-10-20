# What `sync` and archival report about a modification and a withdrawal

**Date:** 2026-09-30
**Tasks:** 1.1
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`. It prints `6a591bdf9d92-dirty`, because the working tree
holds this set's untracked change directories.

Scratch files are under `scratchpad/see-a-modification-the-merge-did-not-take/`. The scripts are
reproduced in the appendices, so the measurement does not depend on that directory surviving. The
whole run was repeated from an empty directory with `run-all.sh` (appendix D), and every number below
is from that run.

## Summary

| Step | What the change states, and what was merged | `sync` | `archive --check` |
| --- | --- | --- | --- |
| 1 | `req-x` `modified`, one word added to its statement. Nothing merged | `req-x` reached, "applied in full" | "ready", `needsSync: false` |
| 2 | as step 1, plus scenario C marked `added`. Nothing merged | `req-x` reached, "applied in full" | "ready", `needsSync: false` |
| 3a | as step 2, plus `req-y` marked `removed`. Nothing merged | `req-x`, `req-y` reached, "applied in full" | "ready", `needsSync: false` |
| 3b | the same change. `req-y` deleted from the accepted specification, which is the correct merge | `req-y` absent | "has not been synced", `needsSync: true`, asks |
| 4 | as step 3a, plus `req-z` marked `unchanged`. `req-y` deleted | `stated` is `req-x`, `req-y`, `req-z` | "has not been synced", `needsSync: true` |
| 5 | the step 4 change. **Every part merged correctly** | `req-y` absent | **"has not been synced"**, `needsSync: true` |
| 6 | the step 4 change. **Nothing merged** | all three reached, **"applied in full"** | **"ready"**, `needsSync: false` |

**Both defects reproduce, and they meet in steps 5 and 6.** A modification nobody merged is
reported as reached. A withdrawal the merge applied is reported as absent. Against the same change,
the untouched accepted specification is called fully applied, and the correctly merged one is called
unsynced.

## 1. The scratch project

`init --tools none`, then one accepted specification and one change, each created with `new document
specification` or `new change` and filled by `write.py` (appendix A). `spec.nix` is raised to `0.1.0`,
because archival's version gate refuses before it reports anything else. The change's one task is set
to `done`, so the only findings are the merge's. Every document is formatted with `htmlspec format
htmlspec`.

The accepted specification, `htmlspec/specs/cap/spec.html`, as the program projects it (`project-model`)
before any merge:

| Requirement | State | Statements | Scenarios |
| --- | --- | --- | --- |
| `req-x` | `unchanged` | "The system SHALL do x." | `scenario-x-a` "Scenario A", `scenario-x-b` "Scenario B" |
| `req-y` | `unchanged` | "The system SHALL do y." | `scenario-y-a` "Scenario Y" |
| `req-z` | `unchanged` | "The system SHALL do z." | `scenario-z-a` "Scenario Z" |

The change's specification, `htmlspec/changes/measure/specification.html`, as projected at step 4:

| Requirement | State | Statements | Scenarios |
| --- | --- | --- | --- |
| `req-x` | `modified` | "The system SHALL do x **quickly**." | `scenario-x-a` `unchanged`, `scenario-x-b` `unchanged`, `scenario-x-c` `added` "Scenario C" |
| `req-y` | `removed` | "The system SHALL do y." | `scenario-y-a` `removed` |
| `req-z` | `unchanged` | "The system SHALL do z." | `scenario-z-a` `unchanged` |

Both documents validate at every step: `validate` reports 0 errors and "the document is accepted".
The warnings are all `review/unverified-requirement`, because the scratch documents carry no
verification.

## 2. A word added, not merged (step 1)

The change states `req-x` `modified`, with the statement "The system SHALL do x quickly.", and scenarios
A and B `unchanged`. The accepted `req-x` still says "The system SHALL do x.".

```
$ htmlspec sync --change measure
1 of 1 requirement(s) this change states have reached the accepted specifications
  reached  req-x
  the change's contract has been applied in full
exit 0

$ htmlspec sync --change measure --json
{
  "absent": [],
  "change": "measure",
  "reached": [
    "req-x"
  ],
  "stated": [
    "req-x"
  ],
  "untakenRenames": []
}
exit 0

$ htmlspec archive measure --check
  0 errors, 1 warning: the document is accepted
ready: archival would archive this change
exit 0            (standard error: "version 0.1.0 follows 0.0.0", "staging 0 runbook release(s) …")

$ htmlspec archive measure --check --json
{
  "archivedTo": null,
  "followsCycle": [],
  "moved": false,
  "needsSync": false,
  "openTasks": [],
  "outOfOrder": false,
  "unmergedRequirements": [],
  "unresolvedPredecessor": null,
  "untakenRenames": []
}
exit 0
```

**`req-x` is reported as reached, and the change as applied in full and ready to archive.** The
statement the change states and the one the accepted specification holds differ by one word, and the
projection holds both texts: `["The system SHALL do x quickly."]` against `["The system SHALL do x."]`.

## 3. A scenario added, not merged (step 2)

The same change adds `scenario-x-c` "Scenario C", marked `added`, to `req-x`. The accepted `req-x`
holds only A and B.

The four outputs are identical to step 1's, byte for byte (`diff` of the two sections of the run's
log differs only in the line "step 1 built"): `req-x` reached, "applied in full", "ready", and
`needsSync: false`. **An unmerged scenario is not seen either.**

## 4. A withdrawal, before and after the merge (steps 3a and 3b)

**Step 3a.** The change also states `req-y`, marked `removed`, with its one scenario marked `removed`.
The accepted specification still holds `req-y`.

```
$ htmlspec sync --change measure
2 of 2 requirement(s) this change states have reached the accepted specifications
  reached  req-x, req-y
  the change's contract has been applied in full
exit 0
```

`--json` gives `"reached": ["req-x", "req-y"]` and `"absent": []`. `archive --check` says "ready:
archival would archive this change", and its payload says `"needsSync": false` and
`"unmergedRequirements": []`. **A withdrawal nobody applied is reached.**

**Step 3b.** `req-y` is deleted from the accepted specification, which is what a correct merge of a
withdrawal does. Nothing else changes.

```
$ htmlspec sync --change measure
1 of 2 requirement(s) this change states have reached the accepted specifications
  reached  req-x
  absent   req-y
exit 0
standard error:
note: 1 requirement(s) appear in no accepted specification. Merge each into the specification that
should hold it, or confirm it was withdrawn deliberately. This command reports the comparison and
never edits an accepted specification

$ htmlspec sync --change measure --json
{ "absent": ["req-y"], "change": "measure", "reached": ["req-x"],
  "stated": ["req-x", "req-y"], "untakenRenames": [] }
exit 0

$ htmlspec archive measure --check
ready with questions: this change has not been synced, and archival would report the findings above and ask
exit 0
standard error:
note: 1 requirement(s) this change states appear in no accepted specification: req-y. This change has
not been synced. Run `htmlspec sync --change measure` to see what remains, apply the change's contract,
or confirm each one was withdrawn deliberately

$ htmlspec archive measure --check --json
{ "archivedTo": null, "followsCycle": [], "moved": false, "needsSync": true, "openTasks": [],
  "outOfOrder": false, "unmergedRequirements": ["req-y"], "unresolvedPredecessor": null,
  "untakenRenames": [] }
exit 0
```

The JSON payloads and the notes on standard error are reflowed here. Their words, members, and values
are as printed. The validation report archival prints before its findings is left out, as it is in
every block below; it is the one `review/unverified-requirement` warning per requirement.

**What archival asks.** Archival without `--check`, run under a pseudo-terminal with `script -q
/dev/null`. The terminal wrapped the question at 80 columns, and it is unwrapped here:

```
? This change has not been synced. Archive anyway, over 1 finding(s)? Answering
no leaves everything as it is, so you can run `htmlspec sync --change measure` first (y/N)
<canceled>
htmlspec: archival stopped at your request. Nothing moved, and the project is as it was. Run
`htmlspec sync --change measure` to see what has not reached the accepted specifications
archive exit 3
```

The pseudo-terminal delivered end-of-input before the answer. The prompt shows that as `<canceled>`,
and archival treats it as a no. Run without a terminal and without `--yes`:

```
htmlspec: archival found the above and cannot ask, because nothing here can answer. Pass --yes to
archive over these findings, or resolve them
archive exit 2
```

A `shasum` of every file under `htmlspec/` and `spec.nix`, outside the store, is identical before and
after both runs. **After a correct merge of the withdrawal, `sync` lists `req-y` as absent, with a note
that opens "Merge each into the specification that should hold it". Archival says the change has not
been synced, asks, and cannot archive without `--yes` where nothing can answer.**

An aside, not this change's subject: at step 3a the change's specification draws
`review/unverified-requirement at req-y`, although `req-y` is marked `removed`.

## 5. What `stated` holds (step 4)

The change also states `req-z`, marked `unchanged`. `req-y` is still deleted from the accepted
specification.

```
$ htmlspec sync --change measure --json
{
  "absent": [
    "req-y"
  ],
  "change": "measure",
  "reached": [
    "req-x",
    "req-z"
  ],
  "stated": [
    "req-x",
    "req-y",
    "req-z"
  ],
  "untakenRenames": []
}
exit 0
```

**`stated` holds every requirement identifier the specification carries, whatever its change state:**
`req-x` (`modified`), `req-y` (`removed`), and `req-z` (`unchanged`). `archive --check` reports
`"unmergedRequirements": ["req-y"]` and `"needsSync": true`.

## 6. Nothing merged, and everything merged (steps 6 and 5)

The step 4 change, measured against the two extremes.

**Step 5, every part merged correctly.** The accepted `req-x` becomes "The system SHALL do x quickly."
with A, B, and C. `req-y` is deleted. `req-z` is untouched.

```
$ htmlspec sync --change measure
2 of 3 requirement(s) this change states have reached the accepted specifications
  reached  req-x, req-z
  absent   req-y
exit 0

$ htmlspec archive measure --check
ready with questions: this change has not been synced, and archival would report the findings above and ask
exit 0          ("needsSync": true, "unmergedRequirements": ["req-y"] under --json)
```

**Step 6, nothing merged.** The accepted specification is put back as it was in section 1.

```
$ htmlspec sync --change measure
3 of 3 requirement(s) this change states have reached the accepted specifications
  reached  req-x, req-y, req-z
  the change's contract has been applied in full
exit 0

$ htmlspec archive measure --check
ready: archival would archive this change
exit 0          ("needsSync": false, "unmergedRequirements": [] under --json)
```

The `sync --json` payload at step 5 is identical to step 4's, although `req-x` was merged in between.
Compared as parsed JSON, the two are equal. **Merging `req-x` changes nothing in the report, and the
correct merge is the one reported as unsynced.**

## 7. The code

`merge_state` is at `crates/htmlspec-cli/src/commands.rs` lines 3545-3600 at HEAD.

```
$ sed -n '3545,3600p' crates/htmlspec-cli/src/commands.rs | grep -c '\.change'
0
```

**The function never reads a requirement's change state.** The one line in it containing the word
"change" is a comment. Every stated identifier goes into `reached` when an accepted specification holds
it, into `absent` when none does, and into `untaken_renames` when it states a rename whose heading was
not taken.

The condition "the contract has arrived in full" is written twice:

- `sync`, line 2475: `if state.absent.is_empty() && state.untaken_renames.is_empty() {`, which prints
  "the change's contract has been applied in full".
- `Findings::needs_sync`, line 3294: `!self.unmerged.is_empty() || !self.untaken_renames.is_empty()`,
  over `Findings`' own copies of those two sets, which `findings_for` (3341-3345) takes from `merge_state`.

## 8. What the instructions tell an agent about a withdrawal

`crates/htmlspec-assets/instructions/`, at HEAD. `state-a-modified-requirement-whole` edits both files
before this change does. The lines below are the ones it leaves untouched.

- `archive.md` line 280, in step 5: "A requirement the change withdrew is removed. A withdrawal is the
  one case where deleting is correct".
- `archive.md` lines 233-235, in step 4, about requirements no accepted specification holds: "Before
  step 5 this is expected — you have not merged yet. After step 5, each remaining one should be a
  requirement the change deliberately withdrew, and you should be able to say which."
- `sync.md` lines 74-86: "Four sets come back", and "`absent` and `untakenRenames` are what you work
  through."
- `sync.md` lines 153-154: "An identifier in `absent` may be there because the requirement was withdrawn —
  the change stated it, the work went another way, and it is correctly absent forever."
- `sync.md` line 168: "`absent` should be empty, or hold only the identifiers you reported as withdrawn".

**The instructions expect a correctly merged withdrawal to be reported as absent, which is what section
4 measured.** The word "withdrawn" covers two cases in them: a requirement the change marks `removed`,
in `archive.md`, and a requirement the change stated whose work went another way, in `sync.md`.

## 9. A one-word modification in this project's archive

`2026-08-31-sync-a-change-into-the-accepted-contract` landed at `ee4301c`. The same commit edits
`openspec/specs/toolchain-packaging/spec.md`, so the accepted text it modified is the one at `ee4301c^`.
Its delta modifies "The command surface names its operations as OpenSpec names them". `paragraphs.py`
(appendix E) splits each requirement's statement at blank lines, normalizes whitespace, as
`state-a-modified-requirement-whole`'s `count.py` does, and compares the two paragraph by paragraph.
It prints the heading, the number of paragraphs before and in the delta, and the words each differing
paragraph gains or loses:

```
$ git show ee4301c^:openspec/specs/toolchain-packaging/spec.md > before.md
$ git show ee4301c:openspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract/specs/toolchain-packaging/spec.md > delta.md
$ python3 paragraphs.py before.md delta.md
== The command surface reports whether a change reached the accepted contract None 6
== The workflow instructions carry a stage for applying a change's contract None 4
== The command surface names its operations as OpenSpec names them 10 10
 para 3 + `sync`,
```

The first two are the change's added requirements, which the accepted text did not hold (`None`).

**The modification is one word in the third of ten statement paragraphs.** Paragraph 3 is the one that
begins "The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`,
`init`, `new`, …". Every other paragraph is identical.

## Appendix A: `write.py`

```python
#!/usr/bin/env python3
"""Replaces a document's main content with the given requirements and its placeholder summary.

Usage: write.py <document> <summary> <requirements-html>
"""
import re
import sys

path, summary, requirements = sys.argv[1], sys.argv[2], sys.argv[3]
text = open(path).read()
body = (
    '<section id="hs-requirements"><h2>Requirements</h2>' + requirements + "</section>"
    '<section id="hs-decisions"><h2>Decisions</h2></section>'
    '<section id="hs-risks"><h2>Risks</h2></section>'
    '<section id="hs-evidence"><h2>Evidence</h2></section>'
    '<section id="hs-verification"><h2>Verification</h2></section>'
)
text = re.sub(r'(<main class="hs-main" id="hs-content">).*?(</main>)',
              lambda m: m.group(1) + body + m.group(2), text, flags=re.S)
text = re.sub(r"<p>Replace this paragraph with what this document is for\..*?</p>",
              "<p>" + summary + "</p>", text, flags=re.S)
open(path, "w").write(text)
```

## Appendix B: `build.sh`

```bash
#!/usr/bin/env bash
# Builds the scratch project for see-a-modification-the-merge-did-not-take, step by step.
# Usage: build.sh <step>   (1, 2, 3a, 3b, 4, 5, 6)
set -u
BIN=target/debug/htmlspec
S=<scratch>/see-a-modification-the-merge-did-not-take
P=$S/proj
W="python3 $S/write.py"

scenario() {  # id heading change-state
  printf '<article class="scenario" id="%s" data-change="%s"><h4>%s</h4><dl><dt>When</dt><dd>%s happens.</dd><dt>Then</dt><dd>%s holds.</dd></dl></article>' "$1" "$3" "$2" "$2" "$2"
}
requirement() {  # id letter change-state statement scenarios...
  local id=$1 letter=$2 state=$3 statement=$4; shift 4
  printf '<section class="requirement" id="%s" data-change="%s"><h3>The %s requirement <span class="hs-id">%s</span></h3><p>%s</p>%s</section>' "$id" "$state" "$letter" "$id" "$statement" "$*"
}

X_A=$(scenario scenario-x-a "Scenario A" unchanged)
X_B=$(scenario scenario-x-b "Scenario B" unchanged)
X_C=$(scenario scenario-x-c "Scenario C" added)
ACCEPTED_X=$(requirement req-x x unchanged "The system SHALL do x." "$X_A$X_B")
ACCEPTED_Y=$(requirement req-y y unchanged "The system SHALL do y." "$(scenario scenario-y-a "Scenario Y" unchanged)")
ACCEPTED_Z=$(requirement req-z z unchanged "The system SHALL do z." "$(scenario scenario-z-a "Scenario Z" unchanged)")
SPEC=$P/htmlspec/changes/measure/specification.html
ACC=$P/htmlspec/specs/cap/spec.html

cd "$P"
case $1 in
  1)
    # The accepted specification holds req-x, req-y, and req-z. The change states req-x
    # modified, with one word added to its statement, and nothing is merged.
    $BIN new document specification htmlspec/specs/cap/spec.html --subject "The accepted capability" >/dev/null 2>&1
    $W "$ACC" "The accepted capability, for measuring the merge comparison." "$ACCEPTED_X$ACCEPTED_Y$ACCEPTED_Z"
    # Archival's version gate refuses before it reports anything else, so the version moves.
    sed -i '' 's/version = "0.0.0"/version = "0.1.0"/' spec.nix
    $BIN new change measure >/dev/null 2>&1
    sed -i '' 's/data-state="todo"/data-state="done"/g' htmlspec/changes/measure/tasks.html
    $BIN new document specification htmlspec/changes/measure/specification.html --subject "The measured change" >/dev/null 2>&1
    $W "$SPEC" "A change that modifies req-x." \
      "$(requirement req-x x modified "The system SHALL do x quickly." "$X_A$X_B")"
    ;;
  2)
    # The same change also adds a scenario to req-x. Still nothing merged.
    $W "$SPEC" "A change that modifies req-x." \
      "$(requirement req-x x modified "The system SHALL do x quickly." "$X_A$X_B$X_C")"
    ;;
  3a)
    # The change also withdraws req-y, stated whole and marked removed. Not merged yet.
    $W "$SPEC" "A change that modifies req-x and withdraws req-y." \
      "$(requirement req-x x modified "The system SHALL do x quickly." "$X_A$X_B$X_C")$(requirement req-y y removed "The system SHALL do y." "$(scenario scenario-y-a "Scenario Y" removed)")"
    ;;
  3b)
    # The withdrawal merged correctly: req-y deleted from the accepted specification.
    $W "$ACC" "The accepted capability, for measuring the merge comparison." "$ACCEPTED_X$ACCEPTED_Z"
    ;;
  4)
    # The change also carries req-z forward, marked unchanged.
    $W "$SPEC" "A change that modifies req-x, withdraws req-y, and carries req-z forward." \
      "$(requirement req-x x modified "The system SHALL do x quickly." "$X_A$X_B$X_C")$(requirement req-y y removed "The system SHALL do y." "$(scenario scenario-y-a "Scenario Y" removed)")$(requirement req-z z unchanged "The system SHALL do z." "$(scenario scenario-z-a "Scenario Z" unchanged)")"
    ;;
  5)
    # The control: req-x merged as the change states it, the word and scenario C included.
    $W "$ACC" "The accepted capability, for measuring the merge comparison." \
      "$(requirement req-x x unchanged "The system SHALL do x quickly." "$X_A$X_B$(scenario scenario-x-c "Scenario C" unchanged)")$ACCEPTED_Z"
    ;;
  6)
    # The accepted specification put back as it was before any merge, against the step 4 change.
    $W "$ACC" "The accepted capability, for measuring the merge comparison." "$ACCEPTED_X$ACCEPTED_Y$ACCEPTED_Z"
    ;;
esac
$BIN format htmlspec >/dev/null 2>&1
echo "step $1 built"
```

## Appendix C: `measure.sh`

```bash
#!/usr/bin/env bash
# Runs the four observations for one step: sync (human and --json) and archive --check
# (human and --json). Standard output and standard error are kept apart.
set -u
BIN=target/debug/htmlspec
cd <scratch>/see-a-modification-the-merge-did-not-take/proj
run() {
  echo "\$ htmlspec $*"
  "$BIN" "$@" >out.txt 2>err.txt; code=$?
  echo "--- stdout"; cat out.txt; echo "--- stderr"; cat err.txt; echo "--- exit $code"; echo
}
run sync --change measure
run sync --change measure --json
run archive measure --check
run archive measure --check --json
rm -f out.txt err.txt
```

## Appendix D: `run-all.sh`

```bash
#!/usr/bin/env bash
# Runs the whole measurement from an empty directory: every step, in order.
set -u
BIN=target/debug/htmlspec
S=<scratch>/see-a-modification-the-merge-did-not-take
P=$S/proj
rm -rf "$P"; mkdir -p "$P"; cd "$P" && git init -q
"$BIN" --version
"$BIN" init --tools none --root "$P" >/dev/null 2>&1; echo "init exit $?"
for step in 1 2 3a 3b; do
  echo "=================== step $step"; "$S/build.sh" $step
  for doc in htmlspec/specs/cap/spec.html htmlspec/changes/measure/specification.html; do
    "$BIN" validate "$doc" 2>&1 | tail -1 | sed "s|^|validate $doc: |"
  done
  "$S/measure.sh"
done
echo "=================== step 3b, archival asked on a terminal, answered with end-of-input"
find htmlspec spec.nix -type f -not -path '*/assets/*' | sort | xargs shasum > "$S/before.sha"
printf 'n\n' | perl -e 'alarm 20; exec @ARGV' script -q /dev/null "$BIN" archive measure 2>&1 | tr -d '\r' | sed 's/\x1b\[[0-9;?]*[A-Za-z]//g'
echo "archive exit ${PIPESTATUS[1]}"
find htmlspec spec.nix -type f -not -path '*/assets/*' | sort | xargs shasum | diff - "$S/before.sha" >/dev/null && echo "project unchanged"
echo "=================== step 3b, archival without a terminal and without --yes"
"$BIN" archive measure; echo "archive exit $?"
find htmlspec spec.nix -type f -not -path '*/assets/*' | sort | xargs shasum | diff - "$S/before.sha" >/dev/null && echo "project unchanged"
for step in 4 5 6; do
  echo "=================== step $step"; "$S/build.sh" $step
  for doc in htmlspec/specs/cap/spec.html htmlspec/changes/measure/specification.html; do
    "$BIN" validate "$doc" 2>&1 | tail -1 | sed "s|^|validate $doc: |"
  done
  "$S/measure.sh"
done
```

## Appendix E: `paragraphs.py`

```python
#!/usr/bin/env python3
"""For each requirement a delta states, compares its statement paragraphs with the accepted text.

Usage: paragraphs.py <accepted-spec.md> <delta-spec.md>
"""
import difflib
import re
import sys


def statement(text, heading):
    m = re.search(r'^### Requirement: ' + re.escape(heading) + r'\n(.*?)(?=^### Requirement:|^## |\Z)',
                  text, re.M | re.S)
    if not m:
        return None
    body = m.group(1).split('#### Scenario')[0]
    return [re.sub(r'\s+', ' ', p).strip() for p in body.split('\n\n') if p.strip()]


before, delta = open(sys.argv[1]).read(), open(sys.argv[2]).read()
for heading in re.findall(r'^### Requirement: (.+)$', delta, re.M):
    a, b = statement(before, heading), statement(delta, heading)
    print('==', heading, None if a is None else len(a), len(b))
    for i, (x, y) in enumerate(zip(a or [], b)):
        if x != y:
            print(' para', i + 1, ' '.join(t for t in difflib.ndiff(x.split(), y.split()) if t[0] in '+-'))
```
