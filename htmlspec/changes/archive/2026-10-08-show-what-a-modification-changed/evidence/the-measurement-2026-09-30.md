# What a reviewer can see of a modification, in OpenSpec and in html-spec, measured against HEAD

**Date:** 2026-09-30
**Tasks:** 1.2 and 1.3, which re-measure this against HEAD before any change, and 7.1, which repeats
section 6 against the built change
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`. It prints `6a591bdf9d92-dirty`, because the working
tree holds this set's untracked change directories. Every `htmlspec` below is this repository's
`./target/debug/htmlspec`, called by its path.
**OpenSpec:** `openspec` 1.13.1, from `PATH`.
**Projection:** `~/projects/html-spec-migration/tools/project-model/target/release/project-model`,
which calls `htmlspec_parse::project` from this repository's `crates/htmlspec-parse`. It is written
`project-model` below.

Scratch files are under `scratchpad/show-what-a-modification-changed/`. The two scripts are
reproduced in the appendices, so the measurement does not depend on that directory surviving.
Nothing in the repository or in the staged migration was written.

## Summary

| Question | Answer |
| --- | --- |
| What does OpenSpec show for the one-word modification, while the change is open? | One unified-diff hunk, `@@ -3,9 +3,9 @@`: the third paragraph as a `-` line and a `+` line, which differ by `` `sync`, ``, with two paragraphs of context on each side. The other seven paragraphs and all 18 scenarios are not shown. |
| And for an ADDED requirement? | The whole requirement, every paragraph and scenario, indented under `ADDED:`. |
| And once the accepted text holds the edit? | `(no textual changes)` under `MODIFIED:`. The ADDED requirements are still printed whole. |
| And once the change is archived? | Nothing. `Unknown item`, exit 1, by its name, its dated name, or `archive/<dated name>`. |
| What does OpenSpec's `show <change>` print without `--diff`? | The change's `proposal.md`, byte for byte, and nothing else. |
| Is there a structured form? | `--json` refused this change with "Change must have a What Changes section", exit 1. With that heading added to a scratch copy of its proposal, the payload carries a `diff` string, the same hunk, on each MODIFIED delta and none on an ADDED one. |
| What does html-spec have? | Nothing. `htmlspec show` is an unrecognized subcommand, exit 2. No crate, instruction, or accepted specification names `show` or `--diff`. |
| How large is a modified requirement a reviewer has to search? | 34 modified requirements in 19 staged changes. Median 7 statement paragraphs and 7 scenarios, maximum 15 paragraphs and 18 scenarios. The one-word case is 10 paragraphs, 491 words, and 18 scenarios, and its edit is one word. |
| Could the view serve the staged migration? | Not as an active change. All 24 staged changes are archived, and `list --json` reports no active change. |
| Does the program already order the archive? | Only runbook snapshots, by recorded project version, in `runbook_release::history`. For changes it takes the greatest recorded version and nothing else. |
| What would a diff of an archived change against today's text show? | For `2026-08-30-finish-the-archival`, one paragraph where the accepted text holds `sync,` and the change does not. That word is the *next* change's edit. finish-the-archival's own edit, a rename and four added scenarios, does not appear. |

## 1. OpenSpec's view, while the change was open

### The tree

```
$ git log --oneline -- openspec/changes/sync-a-change-into-the-accepted-contract \
    openspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract
ee4301c Add the sync operation archival tells you to run (#16)
8ca4b59 Propose a sync operation and progress in the change listing (#15)
```

At `8ca4b59` the change is open. At `ee4301c` it is archived and the accepted specification holds
its edit. Both trees were extracted into the scratch directory, which is not inside a git
repository:

```
$ git archive 8ca4b59 openspec | tar -x -C <scratch>/openspec-at-8ca4b59
$ git archive ee4301c openspec | tar -x -C <scratch>/openspec-at-ee4301c
```

At `8ca4b59`, `openspec/changes/` holds four open changes: `carry-task-progress-into-the-listing`,
`distinguish-a-requirement-from-an-example`, `migrate-an-openspec-project`, and
`sync-a-change-into-the-accepted-contract`.

### The human view

```
$ cd <scratch>/openspec-at-8ca4b59
$ openspec show sync-a-change-into-the-accepted-contract --diff --no-interactive   # exit 0, 153 lines, nothing on stderr
```

Lines 1 to 69 are the change's `proposal.md`, byte for byte (`cmp` of the first 69 lines against the
file reports no difference). Then:

```
Specifications Changed (diffs)

toolchain-packaging

  ADDED: The command surface reports whether a change reached the accepted contract
    ### Requirement: The command surface reports whether a change reached the accepted contract
    
    The command surface SHALL provide an operation that reports, for one change, whether the …
    …                                                    (every paragraph and all 7 scenarios)
    - **THEN** no accepted specification was modified by the program

  ADDED: The workflow instructions carry a stage for applying a change's contract
    …                                                    (every paragraph and both scenarios)

  MODIFIED: The command surface names its operations as OpenSpec names them
    @@ -3,9 +3,9 @@
     The project SHALL provide one command-line program offering, at minimum, a subcommand to validate documents, …
     
     **Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.** This holds for …
     
    -The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, and `--store` carry OpenSpec's spellings for the same reason.
    +The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `sync`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, and `--store` carry OpenSpec's spellings for the same reason.
     
     Fixing a spelling in the contract is not decoration. A requirement that describes an operation by role alone — …
     
     The bootstrap subcommand's subject SHALL be a project and it SHALL require no arguments, …
```

Long lines are cut here with `…`. In the output each paragraph is one line.

**The hunk reproduces what the prompt expected**: `` `sync`, `` is added to the list of spellings,
and that is the whole difference. The hunk is a unified diff over the markdown lines: the changed
paragraph appears twice, whole, once with `-` and once with `+`, and the word is not marked inside
it. A reader finds `` `sync`, `` by comparing two 60-word lines by eye.

**An ADDED requirement is printed whole**, with no diff marks, under `ADDED:`.

### Without `--diff`

```
$ openspec show sync-a-change-into-the-accepted-contract --no-interactive   # exit 0, 70 lines
```

The output is `proposal.md` followed by one empty line. `diff` against the file reports only that
line (`70d69 <`).

### The structured form

```
$ openspec show sync-a-change-into-the-accepted-contract --diff --json --no-interactive   # exit 1
{
  "status": [
    {
      "severity": "error",
      "code": "show_error",
      "message": "Change must have a What Changes section"
    }
  ]
}
```

`--json` without `--diff` gives the same payload and exit 1. This change's proposal has one `## `
heading, `## Why`. So does `carry-task-progress-into-the-listing`'s. The other two open changes have
a `## What Changes` heading, but they only add requirements.

To see the shape OpenSpec gives a MODIFIED requirement, a scratch copy of the tree had
`## What Changes` and one bullet appended to this change's proposal. Nothing else was changed:

```
$ openspec show sync-a-change-into-the-accepted-contract --diff --json --no-interactive   # exit 0
```

| Key | Value |
| --- | --- |
| top level | `id`, `title`, `deltaCount` (3), `deltas`, `root` |
| an ADDED delta | `spec`, `operation`, `description`, `requirement`, `requirements` |
| the MODIFIED delta | the same keys, and `diff` |

The `diff` value is one string, the human view's hunk: `"@@ -3,9 +3,9 @@\n The project SHALL …"`,
with the same `-` and `+` lines. The change's side of the requirement is carried as
`requirement.text`, which holds every statement paragraph (3054 characters for the MODIFIED one), and
`requirement.scenarios[].rawText`, which holds each scenario's markdown. The accepted side appears
only inside `diff`.

### After the merge, before archival

A second scratch copy of the `8ca4b59` tree had `openspec/specs/toolchain-packaging/spec.md` replaced
with the one at `ee4301c`, so the accepted text holds the edit and the change is still open:

```
$ openspec show sync-a-change-into-the-accepted-contract --diff --no-interactive   # exit 0
…
  MODIFIED: The command surface names its operations as OpenSpec names them
    (no textual changes)
```

Both ADDED requirements are still printed whole, although the accepted specification now holds
them.

## 2. OpenSpec's view once the change is archived

In the `ee4301c` tree the change is under `openspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract`.

```
$ openspec show sync-a-change-into-the-accepted-contract --diff --no-interactive                        # exit 1
Unknown item 'sync-a-change-into-the-accepted-contract'. Did you mean: migrate-an-openspec-project, carry-task-progress-into-the-listing, document-template, document-validation, toolchain-packaging?

$ openspec show 2026-08-31-sync-a-change-into-the-accepted-contract --diff --no-interactive             # exit 1
Unknown item '2026-08-31-sync-a-change-into-the-accepted-contract'. Did you mean: …

$ openspec show archive/2026-08-31-sync-a-change-into-the-accepted-contract --diff --no-interactive     # exit 1
Unknown item 'archive/2026-08-31-sync-a-change-into-the-accepted-contract'. Did you mean: …
```

With `--json`, each gives `{"status": [{"severity": "error", "code": "unknown_item", "message": …}]}`
and exit 1. **Once archived, OpenSpec shows nothing, and it does not say the change is archived.**

## 3. html-spec today

```
$ ./target/debug/htmlspec show                         # exit 2
error: unrecognized subcommand 'show'

Usage: htmlspec [OPTIONS] <COMMAND>

For more information, try '--help'.

$ ./target/debug/htmlspec show some-change --diff      # exit 2, the same four lines
$ ./target/debug/htmlspec --json show some-change --diff 2>/dev/null | wc -c
       0
```

The error goes to standard error in every case, and standard output is empty, with `--json` or
without it.

```
$ ./target/debug/htmlspec --help                       # exit 0
…
Commands:
  validate      Validate documents against the contract
  format        Rewrite documents into their canonical form
  assets        Check a project's asset topology
  archive       Archive a completed change
  init          Set up an HTMLSpec project in this repository
  list          List what the project holds
  run           Start a run of a runbook, or check what its runs recorded
  preflight     Report whether a runbook can start in this working environment
  status        Report which of a change's artifacts exist and how much of its work is done
  sync          Report whether a change's contract reached the accepted specifications
  store         Register and manage standalone projects on this machine
  instructions  Print the workflow instructions for a stage
  new           Create a document or a change that validates immediately
  migrate       Bring an existing OpenSpec project into HTMLSpec
  help          Print this message or the help of the given subcommand(s)

Options:
      --json    …
      --store <STORE>    …
```

Nothing names the view anywhere:

```
$ grep -rn -e "htmlspec show" -e "show --diff" -e '"--diff"' -e "Command::Show" crates/     # no output
$ grep -rn -e "show --diff" -e "--diff" openspec/specs/                                       # no output
$ grep -rn "openspec show" crates/htmlspec-assets/instructions/                               # no output
```

`status` and `sync` take the change as `--change <name>`. OpenSpec's `status` takes it the same way
(`openspec status --help`: `--change <id>    Change name to show status for`). OpenSpec's `show`
takes it as a positional `[item-name]` (`openspec show --help`: `Usage: openspec show [options]
[item-name]`, and `--diff  Show per-requirement diffs for delta specs (change)`).

## 4. The size of a modified requirement in the staged migration

```
$ cd ~/projects/html-spec-migration/staged/htmlspec
$ project-model changes/archive/*/specification.html > pm-staged-specifications.json    # exit 0, nothing on stderr
$ python3 sizes.py pm-staged-specifications.json
34 modified requirements in 19 changes
statement paragraphs: min 2, median 7.0, max 15, total 234
scenarios: min 3, median 7.0, max 18, total 286
statement words: min 62, median 364.0, max 893, total 12083
```

A paragraph is a projected statement, and a word is a run of text between whitespace in it. The
largest, by paragraphs plus scenarios:

| Paragraphs | Scenarios | Words | Change | Requirement |
| --- | --- | --- | --- | --- |
| 15 | 18 | 893 | `2026-08-24-pass-the-description-through` | `req-bootstrapping-installs-agent-instructions-for-the-tools-a-project-uses` |
| 10 | 18 | 490 | `2026-08-30-finish-the-archival` | `req-a-single-command-exposes-validate-format-and-init` |
| 10 | 18 | 491 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `req-a-single-command-exposes-validate-format-and-init` |
| 11 | 14 | 607 | `2026-08-21-distinguish-command-names-from-skill-names` | `req-bootstrapping-installs-agent-instructions-for-the-tools-a-project-uses` |
| 14 | 10 | 489 | `2026-09-02-state-a-changes-place-in-a-sequence` | `req-archival-produces-the-specification-and-then-updates-the-accepted-contract` |

The smallest is 2 paragraphs, 62 words, and 4 scenarios. Appendix A prints every row.

**The one-word case is 10 paragraphs, 491 words, and 18 scenarios, all 18 marked `unchanged`.** It is
the only one of the 34 whose scenarios are all `unchanged` (checked by a second pass over the same
file), so its whole edit is inside the statements, and section 1 shows it is one word.

The 286 scenarios inside the 34 requirements are marked: 176 `unchanged`, 95 `added`, 13 `modified`,
and 2 `removed`. Across all 24 specifications, requirements are 137 `added`, 34 `modified`, 1
`removed`, and none `unchanged`.

**All 24 staged changes are archived.** `ls htmlspec/changes/` lists only `archive`, and:

```
$ cd ~/projects/html-spec-migration/staged
$ htmlspec list --json
{
  "changes": [],
  "followsCycles": []
}
```

## 5. What the program already derives

**The pairing of a change's requirement with an accepted one** is in `merge_state`
(`crates/htmlspec-cli/src/commands.rs:3545`). It reads every document beneath `project.specs()`,
builds `held`, a map from each accepted requirement's identifier to its heading names (3561), and
looks each stated identifier up in it (`held.get(id)`, 3590). `sync` and archival's `findings_for`
both call it. It keeps heading names only, not content, at HEAD.

**A change is resolved by `resolve_change` (939) and `resolve_change_in` (1010).** A path that is a
directory is taken as the change, wherever it is. A name is looked up only under `changes/`. So an
archived change is reachable by its path and not by its name. In the staged tree:

```
$ cd ~/projects/html-spec-migration/staged
$ htmlspec sync --change htmlspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract   # exit 0
3 of 3 requirement(s) this change states have reached the accepted specifications
  reached  req-the-command-surface-reports-whether-a-change-reached-the-accepted-contract, req-the-workflow-instructions-carry-a-stage-for-applying-a-change-s-contract, req-a-single-command-exposes-validate-format-and-init
  the change's contract has been applied in full

$ htmlspec sync --change 2026-08-31-sync-a-change-into-the-accepted-contract                            # exit 2
htmlspec: no change named 2026-08-31-sync-a-change-into-the-accepted-contract in …/staged/htmlspec/changes, and no directory at 2026-08-31-sync-a-change-into-the-accepted-contract
```

**The only order the program derives over the archive is for runbooks.** `runbook_release::history`
(`crates/htmlspec-cli/src/runbook_release.rs:1052`) reads each archived change's `VERSION` and its
`runbooks.json` inventory, and sorts the runbook snapshots by recorded project version, then by
directory name (1162). `predecessor` (1209) takes the last snapshot for one runbook name. For
changes, `last_archived_version` (`commands.rs:4647`) reads every archived `VERSION` and takes
`.max()`, which is one version, not an order. `sequence.rs` resolves a stated predecessor by name,
with the leading date removed, and orders nothing. No function reads archived changes'
specifications in any order.

## 6. A prototype of the pairing, on the one-word case

`prototype.py` (appendix B) is **not the program**. It is a Python sketch of the pairing this change's
design proposes, run on `project-model` output, to check that the proposal finds what OpenSpec found.
It aligns statement paragraphs by equality, pairs what is left where each shares at least half its
words with the other, and marks the words each side holds that the other does not. It compares
scenarios as texts, as `see-a-modification-the-merge-did-not-take` compares them.

The staged specification of `2026-08-30-finish-the-archival` states the requirement as it stood
before `sync-a-change-into-the-accepted-contract`, and no change between them modified it. Taking it
as the accepted side:

```
$ cd ~/projects/html-spec-migration/staged/htmlspec
$ project-model specs/*/spec.html changes/archive/*/specification.html > pm-staged-all.json
$ python3 prototype.py pm-staged-all.json \
    changes/archive/2026-08-30-finish-the-archival/specification.html \
    changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract/specification.html \
    req-a-single-command-exposes-validate-format-and-init
paragraph 3 (accepted paragraph 3), 48 words in common
  the accepted text holds: The spellings are therefore fixed rather than left to an implementer: validate, archive, init, new, list, status, instructions, and store name the operations OpenSpec also provides. format and assets name operations it does not. The selectors --json, --specs, --change, and --store carry OpenSpec's spellings for the same reason.
  the change states:       The spellings are therefore fixed rather than left to an implementer: validate, archive, init, new, list, status, {+sync,+} instructions, and store name the operations OpenSpec also provides. format and assets name operations it does not. The selectors --json, --specs, --change, and --store carry OpenSpec's spellings for the same reason.
statements: 10 accepted, 10 in the change, 9 held by both
scenarios: 18 accepted, 18 in the change, 18 held by both
```

**One paragraph pair and one marked word, which is OpenSpec's hunk.** The projected text has no
backticks, because `<code>` is presentational markup and the projection keeps its text only.

### The same comparison, made against today's text for an archived change

Against the staged accepted `specs/toolchain-packaging/spec.html`, the sync change's statement
compares equal: `statements: 10 accepted, 10 in the change, 10 held by both` and `scenarios: 18
accepted, 18 in the change, 18 held by both`. Its edit is not visible, because the accepted text
already holds it.

`2026-08-30-finish-the-archival`, compared with the same file:

```
$ python3 prototype.py pm-staged-all.json specs/toolchain-packaging/spec.html \
    changes/archive/2026-08-30-finish-the-archival/specification.html \
    req-a-single-command-exposes-validate-format-and-init
paragraph 3 (accepted paragraph 3), 48 words in common
  the accepted text holds: … list, status, [-sync,-] instructions, and store name …
  the change states:       … list, status, instructions, and store name …
statements: 10 accepted, 10 in the change, 9 held by both
scenarios: 18 accepted, 18 in the change, 18 held by both
```

**The only difference shown is the next change's word.** finish-the-archival's own edit does not
appear. Its staged requirement carries `renamedFrom` "A single command exposes validate, format,
bootstrap, and create", and four scenarios it marks `added`: `scenario-an-operation-openspec-also-provides`,
`scenario-the-selectors-carry-openspec-s-spellings`, `scenario-an-operation-openspec-does-not-provide`,
and `scenario-a-renamed-operation-breaks-the-installed-instructions`. Today's text holds all four and
the new heading, so they compare equal. Compared against today's text, an archived change shows
either nothing or what later changes did, reversed.

## Appendix A: `sizes.py`

```python
#!/usr/bin/env python3
"""Sizes of every requirement a staged change's specification marks `modified`.

Usage: sizes.py <project-model output>

A requirement's size is its projected statement paragraphs and its scenarios. The projection
is the program's own (project-model, built at HEAD), so a paragraph is what the model calls a
statement, and a scenario is a scenario unit.
"""
import json
import statistics
import sys

model = json.load(open(sys.argv[1]))
rows = []
for path, doc in sorted(model.items()):
    change = path.split('/')[2]
    for r in doc.get('requirements', []):
        if r.get('change') != 'modified':
            continue
        scen = r.get('scenarios', [])
        states = {}
        for s in scen:
            states[s.get('change', 'unchanged')] = states.get(s.get('change', 'unchanged'), 0) + 1
        rows.append((change, r.get('id'), len(r.get('statements', [])), len(scen), states,
                     sum(len(p.split()) for p in r.get('statements', []))))

print(f"{len(rows)} modified requirements in {len({r[0] for r in rows})} changes")
paras = [r[2] for r in rows]
scens = [r[3] for r in rows]
words = [r[5] for r in rows]
for name, xs in (('statement paragraphs', paras), ('scenarios', scens), ('statement words', words)):
    print(f"{name}: min {min(xs)}, median {statistics.median(xs)}, max {max(xs)}, total {sum(xs)}")
print()
for r in sorted(rows, key=lambda r: (-r[2] - r[3], r[0])):
    print(f"{r[2]:3d} {r[3]:3d} {r[5]:5d}  {r[0]}  {r[1]}  {r[4]}")
```

## Appendix B: `prototype.py`

```python
#!/usr/bin/env python3
"""A prototype of the pairing this change's design proposes, run on projected text.

Usage: prototype.py <project-model output> <earlier spec path> <later spec path> <requirement id>

Not the program. It pairs one requirement's statement paragraphs by equality (a longest common
subsequence), then pairs what is left between two equal paragraphs in order where at least half
the words of each are common, and marks the words each side holds that the other does not.
Scenarios are compared as texts (heading name, given, when, then), as
see-a-modification-the-merge-did-not-take compares them.
"""
import difflib
import json
import sys

model = json.load(open(sys.argv[1]))
earlier, later, rid = sys.argv[2], sys.argv[3], sys.argv[4]


def requirement(path):
    return next(r for r in model[path]['requirements'] if r.get('id') == rid)


def scenario_text(s):
    title = s.get('title', '')
    if s.get('id') and title.endswith(s['id']):
        title = title[: -len(s['id'])].strip()
    return (title, tuple(s.get('given', [])), tuple(s.get('when', [])), tuple(s.get('then', [])))


def words(a, b):
    sm = difflib.SequenceMatcher(a=a.split(), b=b.split(), autojunk=False)
    common = sum(block.size for block in sm.get_matching_blocks())
    marked_a, marked_b = [], []
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        x, y = ' '.join(a.split()[i1:i2]), ' '.join(b.split()[j1:j2])
        if tag == 'equal':
            marked_a.append(x); marked_b.append(y)
        else:
            if x: marked_a.append('[-' + x + '-]')
            if y: marked_b.append('{+' + y + '+}')
    return common, ' '.join(marked_a), ' '.join(marked_b)


acc, chg = requirement(earlier), requirement(later)
A, B = acc['statements'], chg['statements']
sm = difflib.SequenceMatcher(a=A, b=B, autojunk=False)
held = 0
for tag, i1, i2, j1, j2 in sm.get_opcodes():
    if tag == 'equal':
        held += i2 - i1
        continue
    for k in range(max(i2 - i1, j2 - j1)):
        a = A[i1 + k] if i1 + k < i2 else None
        b = B[j1 + k] if j1 + k < j2 else None
        if a is not None and b is not None:
            common, ma, mb = words(a, b)
            if common * 2 >= len(a.split()) and common * 2 >= len(b.split()):
                print(f'paragraph {j1 + k + 1} (accepted paragraph {i1 + k + 1}), {common} words in common')
                print('  the accepted text holds:', ma)
                print('  the change states:      ', mb)
                continue
        if a is not None:
            print(f'accepted paragraph {i1 + k + 1} only:', a[:100])
        if b is not None:
            print(f'paragraph {j1 + k + 1} only in the change:', b[:100])
print(f'statements: {len(A)} accepted, {len(B)} in the change, {held} held by both')

sa = [scenario_text(s) for s in acc['scenarios']]
sb = [scenario_text(s) for s in chg['scenarios'] if s.get('change') != 'removed']
both = sum(min(sa.count(t), sb.count(t)) for t in set(sa) | set(sb))
print(f'scenarios: {len(sa)} accepted, {len(sb)} in the change, {both} held by both')
```

The prototype pairs leftover paragraphs by position within a gap, which is simpler than the design's
second alignment. The two agree whenever a gap holds one paragraph on each side, as both cases above
do.
