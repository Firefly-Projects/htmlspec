# The working folder, revised and run

**Date:** 2026-10-08
**Tasks:** 5.1 to 5.6, 6.1, and 6.2. The section "For the assembled stack" is the preparation for
6.3 to 6.5, which wait for the three changes below this one.
**Binary:** this change's build, `htmlspec 0.24.0 (b768860d5b82-dirty)`.

The working folder is `~/projects/html-spec-migration/`, outside the repository and not a git
repository. Before the first edit, it was copied without `staged/` and `tools/*/target/` to
`scratchpad/migrate-into-the-extended-vocabulary/working-folder-before/`, with a `sha256` of each of
its 162 files, and the staged tree's 291 files were hashed. **`staged/` was never written.** Every
run that stages anything ran over a scratch copy, and the staged tree's hashes match after every
step below. `tools/project-model` was used as built and not rebuilt.

## The source the plan was written from

```
$ git archive 6a591bd openspec | tar -x -C <scratch>/source              # 24 archives
$ htmlspec migrate survey --json <scratch>/source > survey-6a591bd.json
entities equal: True 301
counts equal: True
greatestVersion equal: True
sharedVersions equal: True
configuration equal: True
changes equal apart from configuration: True
with configuration: 15 follows: [('2026-09-26-carry-a-runbook-to-another-machine', 'log-a-runbook-run'),
  ('2026-09-26-log-a-runbook-run', 'establish-a-runbook'),
  ('2026-09-29-keep-evidence-out-of-document-discovery', 'migrate-an-openspec-project')]
keys seen: ['created', 'follows', 'schema']
```

**The new survey of `6a591bd` equals the reviewed `survey.json` in every field but two**: each change
gains `configuration`, and `root` names the extraction rather than the repository. No script reads
`root`. This payload is now the working folder's `survey.json`, so the coverage gate still compares
the reviewed plan's 301 items with the same 301 entities.

## 5.1 `make_decisions.py`

- `follows:` is read from each change's `configuration` in `survey.json`. `change-config.tsv` is no
  longer read, and the script stops with a message if the payload carries no configuration.
- `EVIDENCE_REASON` no longer says "The destination has no field for a start date". It gives the
  project's refusal of 2026-10-01, with design decision 11 named.
- No entity restates its date. Each says instead: "The carried file is the only record of that day:
  the archive directory's name records the day the change was archived, which is a different fact."
- A decision or convention may carry dated amendments, rendered after its reviewed text as
  "Amended on 2026-10-08, by `migrate-into-the-extended-vocabulary`, for the plan's next review". The
  reviewed question and answer of every decision are unchanged. Five carry an amendment: decision 1
  (the reason for carrying the file, and `FOLLOWS` from the survey), decision 2 (a scenario carried
  forward takes its accepted counterpart's identifier), and conventions 2, 4, and 7 (design
  decisions 5 and 3).
- The rules-table row now carries `.openspec.yaml` to `evidence/.openspec.yaml`, with `FOLLOWS`
  written from its `follows:`, citing this change's configuration requirement and artifact-kinds' "A
  change states the change it follows". It used to say "folded or dropped" and "a `FOLLOWS` file, or
  none", which contradicted decision 1.
- **Decision 2's amendment needs a record in the plan, and now has one.** The script reads
  `replay.json` and adds a `scenario:` note wherever a scenario a change carries forward has a
  qualified accepted counterpart and no twin in its own document. It adds exactly one, on
  `record-a-renamed-requirement`'s toolchain-packaging delta: "An invalid specification", `unchanged`,
  now `scenario-the-command-surface-reports-whether-a-change-reached-the-accepted-contract-an-invalid-specification`.
  So `replay.py` now runs before `make_decisions.py`.

The regenerated `decisions.json`: `grep -c` finds 0 of "a `<code>FOLLOWS</code>` file, or none",
0 of "no field for a start date", 0 of "is when the change was started", 5 of "Amended on
2026-10-08", and 15 of "only record of that day". It holds 17 entity decisions, as before, and 7
scenario notes, against 6.

## 5.2 `stage_carry.py`

`FOLLOWS` is written from the survey's payload, at `htmlspec/changes/archive/<name>/FOLLOWS` (or
`htmlspec/changes/<name>/FOLLOWS` for an active change), for each change whose `configuration` states
`follows`. No `.openspec.yaml` is opened. Each `.openspec.yaml` is still carried byte-identical to
`evidence/.openspec.yaml` by the plan's rule. A target holding the same bytes is left as it is. One
holding other bytes is refused, and so is a `FOLLOWS` in a change the survey gives no predecessor.
`carried.json` records every carried file and every `FOLLOWS`, as before.

```
== over a copy of the staged tree, twice
169 entities carried byte-identical, 3 FOLLOWS, 0 file(s) written
169 entities carried byte-identical, 3 FOLLOWS, 0 file(s) written
== over a copy without the 3 FOLLOWS and 15 configuration files, twice
169 entities carried byte-identical, 3 FOLLOWS, 18 file(s) written
169 entities carried byte-identical, 3 FOLLOWS, 0 file(s) written
…/2026-09-26-carry-a-runbook-to-another-machine/FOLLOWS: log-a-runbook-run
…/2026-09-26-log-a-runbook-run/FOLLOWS: establish-a-runbook
…/2026-09-29-keep-evidence-out-of-document-discovery/FOLLOWS: migrate-an-openspec-project
staged-b equals staged-proof byte for byte
staged-proof equals the staged tree
```

Three planted faults, each refused with exit 1: a carried `VERSION` with a line appended ("it holds
other bytes"), a `FOLLOWS` in `establish-a-runbook` ("the survey reports no predecessor"), and a
`FOLLOWS` holding another value. **The `carried.json` the revised script writes equals the working
folder's own, byte for byte, so it was not rewritten.** The staged tree needs nothing from the
revision: the scratch copy the script ran over is identical to it.

## 5.3 `replay.py`

```
scenario states across every delta: {'added': 594, 'unchanged': 178, 'removed': 2, 'modified': 13}
0 anomalies
0 disagreement(s) between the replay and the accepted specifications
$ python3 replay_vs_history.py <repository> <folder>
0 disagreement(s) between the replay and the accepted specs at each archival commit
```

- All 34 modified units carry `previous_statement`, and 33 of them changed their statement text.
- All 191 modified or unchanged scenarios in them carry `previous_text`.
- `separate-artifact-kinds`' toolchain-packaging unit lists **seven** scenarios. Five are stated by
  its block, and "Unknown subcommand" and "Discoverable help" are `unchanged`, with
  `kept_by_merge: true` and their `text`. The unit's own `kept_by_merge` list stays, because
  `replay_vs_history.py` reads it.
- `unchanged` rose from 176 to 178, by those two. Setting the new fields aside, every other unit in
  `replay.json` equals the old one, and so does the archive order.

## 5.4 `build_packets.py`

Each modified requirement carries `previous_statement_text`, and each scenario that existed before
carries `previous_text`. A kept scenario carries its `text`, and a `why` saying the merge kept it. A
withdrawn scenario's `why` now says the accepted specification at the archival commit no longer held
it. Every scenario that is not `added` takes its accepted counterpart's identifier where the accepted
specification holds one, and the script reports any scenario whose plan-derived identifier differs
from that.

```
37 packets written
0 identifier problem(s)
finish-the-archival modified: [('req-archival-requires-the-project-version-to-advance', True, 7, 10),
  ('req-a-single-command-exposes-validate-format-and-init', True, 14, 18),
  ('req-archival-produces-the-specification-and-updates-the-accepted-contract', True, 4, 6)]
non-added scenarios with an accepted counterpart: 191 differing: []
scenario identifiers that moved from the old packets:
  separate-artifact-kinds … 'Unknown subcommand', None -> 'scenario-unknown-subcommand'
  separate-artifact-kinds … 'Discoverable help', None -> 'scenario-discoverable-help'
  record-a-renamed-requirement … 'An invalid specification',
    'scenario-an-invalid-specification' -> 'scenario-the-command-surface-reports-whether-a-change-reached-the-accepted-contract-an-invalid-specification'
```

**`finish-the-archival`'s packet holds previous text for its three modified requirements**, and no
scenario that is not `added` carries an identifier other than its accepted counterpart's. The 191
are the 189 the measurement found, and the two kept scenarios. Setting the new text fields aside,
only two packets differ from the old ones: `separate-artifact-kinds` and
`record-a-renamed-requirement`. All 24 change packets were rewritten, because each requirement now
carries `previous_statement_text`. The 13 accepted-specification packets are unchanged.

## 5.5 `brief-draft.md`

`grep -c` finds 0 of "Give it no `data-refs`", 0 of "references resolve within one document only",
and 0 of "`passed`, `failed`, `blocked`, or". `brief-draft.pilot.md` is byte-identical to its backup.
Lines 91-93 and the creation-record line stay as they were. The changes:

- **Paths:** an assignment may name another staged root, program, source, or report directory, which
  is how the proof's drafts are pointed at a scratch copy and the stacked binary.
- **Decision 2:** amended, as `make_decisions.py` amends it.
- **Convention 2:** resolved as design decision 5 states, with a note saying it used to contradict
  the rule for tasks. **Convention 4:** the archival commit decides, and the packet carries the
  previous text. **Convention 7:** a scenario the merge kept.
- **Summaries:** a summary does not narrate which words a modification changed.
- **Links:** a link to another document is not validated, and a reference is. The `data-refs`
  prohibition is gone. A new section, "References", states the rule, the places a change's documents
  reach, what stays a link alone, and what the review flags.
- **Risks:** the shape rule, `data-response` only where the source states a settlement, the default,
  a headed risk with neither staying prose, and "its response, never its disposition".
- **Tasks:** the four states as project-migration's task requirement gives them, the strike before
  or after the number, a deviation in the author's words where the task's text carries the account
  and opening "Migration note:" otherwise, no `del` around a task's words, and no reason invented.
  It also says that a task's `deviation` is not the build-differs-from-delta classes, which the
  brief also calls deviations.
- **The change's specification:** a new bullet, "A modified requirement", on stating it whole and
  marking the changed words against the packet's previous text, both ways, with a rewritten
  paragraph deleted and added whole.
- **Verifications:** `partial`, with `unchecked` paragraphs in the scenario's words, and the
  boundary against a check of something nearby.
- **Before you finish** and **Your report:** the marks checked both ways, every reference linked, and
  a mapped distinction not reported as open.

## 5.6 `check_draft.py`, and its self-test

`check_draft.py` takes `--staged`, `--source`, `--htmlspec`, and `--project-model`, defaulting to the
paths it had, so a proof run names a scratch copy, the `6a591bd` source, and a stacked build. The
checks added:

- **Tasks.** The strike is read before or after the task number, and a struck or `[~]` task is
  expected `dropped`. A deviation on a task the source did not tick, or struck, is reported, and so
  is a task whose source text carries "**Amended during implementation.**" or "**Widened during
  implementation.**" with no deviation.
- **Marks.** Each requirement and scenario marked `modified` is read twice from the HTML: with `del`
  unwrapped and `ins` dropped, each piece of the packet's previous text must be found, and with `ins`
  unwrapped and `del` dropped, each piece of the delta's text. A modified unit whose text changed and
  carries no `ins` or `del` is reported.
- **References.** In a task or decision, a link outside every migration note to a unit the document
  reaches, by identifier in its change's specification and then the accepted specifications, with
  no `data-refs` for it, is reported. So is a `data-refs` entry no link in its unit points at.
- **`FOLLOWS`.** It must hold the survey's predecessor where there is one, and be absent otherwise.
- **One pre-existing false finding removed.** "FROM: ### Requirement:" and "TO: ### Requirement:",
  the scaffolding of a RENAMED pair, were reported as source sentences missing from
  `finish-the-archival`, and the backed-up checker reports them too. They are now structural, as
  "Requirement:" already was. Without this, 6.3 could never reach 0 findings.

**Each check was proved by planting its fault.** `check_draft_selftest.py`, new in the working
folder, copies the staged tree to a temporary directory for each case, plants one fault, runs the
checker on one packet, and asserts the finding, or its absence where nothing was planted. Against
the working folder and its staged tree:

```
ok   a strike after the task number is read, so the sync packet's task 4.5 draws no finding
ok   a struck-through task carried as done is reported
ok   a deviation on a task the source struck through is reported
ok   a task whose source says it was amended and states no deviation is reported, and a deviation clears it
ok   a modified requirement whose text changed and carries no mark is reported
ok   correct marks draw no marks finding
ok   a word marked inserted that the previous text held is reported, in the previous direction only
ok   a word marked deleted that the delta keeps is reported, in the current direction only
ok   a link to a reachable unit with no data-refs is reported, and the reference clears it
ok   a data-refs entry no link in its unit points at is reported
ok   a link inside a migration note asks for no reference
ok   FOLLOWS that holds the survey's predecessor draws no finding
ok   a missing FOLLOWS is reported
ok   FOLLOWS holding another value is reported
ok   FOLLOWS in a change the survey gives no predecessor is reported
15 of 15 cases pass
real 60.59   user 53.54
staged tree unchanged
```

The marks cases use `sync-a-change-into-the-accepted-contract`, whose one modification added
`<code>sync</code>, ` to a list. Marked `<ins><code>sync</code>, </ins>`, it draws no finding; an
`ins` around `validate` fails only the previous direction, and a `del` around it only the current
one. The amended-task case uses `distinguish-a-requirement-from-an-example`, whose tasks 3.2, 3.4,
3.6, and 4.3 carry their author's account, because no pilot holds one.

On the sync packet the revised checker no longer reports `task-4-5`. On the old drafts it reports
what the revision asks for and they predate: links with no `data-refs`, modified units with no marks,
and, in `finish-the-archival` and `sync`, a previous sentence the unmarked draft no longer holds.

## 6.1 The built change

```
keys on each change: ['archived', 'configuration', 'deltaSections', 'name', 'version']
every change carries configuration: True
presence agrees with the files: True []
archived with a file: 21 follows: 6 none: 9
{"archived": true, "configuration": {"follows": "establish-a-runbook", "keys": ["schema", "created", "follows"]},
 "deltaSections": ["ADDED Requirements", "MODIFIED Requirements"], "name": "2026-09-26-log-a-runbook-run", "version": "0.14.0"}
created value anywhere in changes: False
```

`migrate.md`'s table now has rows at lines 85 (`## MODIFIED Requirements`), 86
(`## REMOVED Requirements`), 88 (a task's checkbox), and 90 (`.openspec.yaml`), twelve in all.
`grep -c`: `` `ins` `` 2, `` `del` `` 3, `deviation` 4, `` `partial` `` 1, `` `unchecked` `` 1,
`` `data-refs` `` 1, `` `data-response` `` 1, `FOLLOWS` 3, `.openspec.yaml` 2, `created:` 3,
`REMOVED` 1, `MODIFIED` 3, and `disposition` 6: the four plan-entity uses, and the two in the risk
part that say a risk's response is not one.

## 6.2 The routines against the plan's source

In a scratch copy of the working folder, against the `6a591bd` extraction:

| Step | Result |
| --- | --- |
| `migrate survey --json` | 301 entities, 24 changes, 15 configurations, 3 stating `follows` |
| `replay.py` | 0 anomalies, 0 disagreements |
| `replay_vs_history.py`, against the repository | 0 disagreements |
| `make_decisions.py` | 17 entity decisions, 5 decisions, 8 rules |
| `build_plan.py`, with a tasks template `new document` rendered | 301 task units: 169 carried, 85 converted, 46 folded, 1 dropped, which are the reviewed plan's; the plan validates with no diagnostics |
| `migrate promote plan.html --survey survey.json`, no `--staged` | `301 surveyed entities, 301 plan items`, "the coverage check passed", exit 0 |
| `stage_carry.py` | 169 carried, among them all 15 `.openspec.yaml` files, each to `evidence/.openspec.yaml`, and 3 `FOLLOWS` |
| `build_packets.py` | 37 packets, 0 identifier problems |

**Every expected result holds.** The same outputs, regenerated in the working folder itself, are
byte-identical to the scratch run's. The reviewed `plan/plan.html` was not rebuilt in place: its
amendments go to the contributor at the plan's next review, which is part of the re-stage, and a
copy built with them is in the scratch run.

## What changed in the working folder

| File | What |
| --- | --- |
| `make_decisions.py`, `stage_carry.py`, `replay.py`, `build_packets.py`, `brief-draft.md`, `check_draft.py` | edited, as above |
| `check_draft_selftest.py` | new: the checker's planted-fault test |
| `survey.json` | regenerated from `6a591bd` with this change's survey |
| `replay.json`, `decisions.json` | regenerated |
| `packets/*.json` | the 24 change packets regenerated; the 13 accepted ones unchanged |
| `carried.json`, `plan/plan.html`, `brief-draft.pilot.md`, `change-config.tsv`, `staged/` | unchanged. `change-config.tsv` is no longer read by anything |

## For the assembled stack: tasks 6.3 to 6.5

They wait for `record-a-task-done-differently`, `record-a-partial-verification`, and
`record-how-a-risk-was-settled`: a draft carrying a deviation, `partial`, or `data-response` fails
validation without them. What they need is ready:

- the packets, the brief, and the checker in the working folder, revised;
- `scratchpad/migrate-into-the-extended-vocabulary/staged-proof/`, a copy of the staged tree over
  which the revised `stage_carry.py` has run (it wrote nothing, and the copy equals `staged/`);
- `scratchpad/migrate-into-the-extended-vocabulary/source/`, the `6a591bd` extraction.

If the scratch directory is gone, recreate both:

```
S=<scratch>; M=~/projects/html-spec-migration; STACK=<the assembled stack's worktree>
mkdir -p $S/source && git -C $STACK archive 6a591bd openspec | tar -x -C $S/source
cp -a $M/staged $S/staged-proof
python3 $M/stage_carry.py $S/source $M $S/staged-proof      # 169 carried, 3 FOLLOWS, 0 written
```

Build the program and the projection tool against the stack, without touching the working folder's
tool:

```
(cd $STACK && nix develop -c cargo build)
mkdir -p $S/project-model && cp -R $M/tools/project-model/Cargo.toml $M/tools/project-model/src $S/project-model/
sed -i '' "s#crates/htmlspec-parse#$STACK/crates/htmlspec-parse#" $S/project-model/Cargo.toml
(cd $S/project-model && nix develop $STACK -c cargo build --release)
H=$STACK/target/debug/htmlspec; PM=$S/project-model/target/release/project-model
```

Then:

- **6.3 and 6.4.** One drafter per packet, under `brief-draft.md`, told to use `$S/staged-proof` as S,
  `$H` as the program, `$S/source` as the source, and a report directory of its own. Each first
  removes its packet's drafted documents from the copy. The packets are
  `spec-runbook-portability`, `2026-09-26-carry-a-runbook-to-another-machine`,
  `2026-08-30-finish-the-archival`, `2026-08-13-content-address-runtime-assets`,
  `2026-08-14-separate-artifact-kinds`, and `2026-09-06-distinguish-a-requirement-from-an-example`.
  Check with:

  ```
  python3 $M/check_draft.py $M --staged $S/staged-proof --source $S/source --htmlspec $H --project-model $PM <the six packets>
  $H migrate promote $M/plan/plan.html --survey $M/survey.json --staged $S/staged-proof --into $S/dest --check
  python3 $M/check_draft_selftest.py $M $S/staged-proof $S/source $H $PM
  ```

  6.3's second check rebuilds one of `finish-the-archival`'s modified requirements in a scratch
  project made with `$H init`: the accepted specification holding the text the change modified,
  from the packet's `previous_statement_text` and `previous_text`, and an active change whose
  specification holds the re-drafted requirement with its marks. `$H show <change> --diff` then
  gives the view's word marks to compare with the draft's.
- **6.5.** The same `check_draft.py` command over the other 31 packets, with the findings tallied by
  kind.

**A preliminary 6.5, run now with this branch's binary**, over the 31 other drafts as they stand:
12 draw no finding (11 accepted specifications and `report-in-the-contributors-terms`). The rest
draw 123 findings. 116 are of kinds the revision expects of drafts that predate it, and 7 were reported before it:

| Kind | Findings | Packets |
| --- | --- | --- |
| a link to a reachable unit with no `data-refs` | 45 | 9 |
| a modified unit whose text changed, with no mark | 39 | 16 |
| previous text not given with `del` unwrapped | 30 | 7 |
| source sentences not found, or scenario entries differing, each in the working folder's `check-all.txt` before this change: `distinguish-command-names-from-skill-names`' two class A rewrites, `project-bootstrap`'s placeholder Purpose, which decision 3 replaced, and three in `2026-08-10-html-spec` | 7 | 3 |
| `record-a-renamed-requirement`'s "An invalid specification", whose identifier the packet now qualifies | 2 | 1 |

No draft draws a `FOLLOWS` finding, a task-state finding, or a deviation finding.
