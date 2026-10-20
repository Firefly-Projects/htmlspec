# A change's creation date, measured before this change

**Copied on 2026-10-01.** This was measured for `record-when-a-change-was-created`, a change
withdrawn on 2026-10-01 because no command needs a creation date. "This change" and the task
numbers below are that change's. It is kept in `migrate-into-the-extended-vocabulary` because that
change's decision about `.openspec.yaml` rests on it (its design decision 11). Below this note, the
file is as it was measured.

**Date:** 2026-09-30
**Tasks:** 1.2 and 1.3. Task 7.1 repeats sections 1 and 2 against the built change.
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`, built at HEAD. `--version` prints
`6a591bdf9d92-dirty`. The marker comes from `git status --porcelain`, whose 13 entries are all
untracked (`??`), so no tracked file differs from HEAD.

## 1. `.openspec.yaml` across the archive

```
$ ls -d openspec/changes/archive/*/ | wc -l
24
$ ls openspec/changes/archive/*/.openspec.yaml | wc -l
15
```

Each file's keys and values, read with `grep '^created:'` and `grep '^follows:'`:

| Archive | `created:` | `follows:` | Bytes |
| --- | --- | --- | --- |
| `2026-08-10-html-spec` | 2026-08-04 | | 40 |
| `2026-08-13-content-address-runtime-assets` | 2026-08-10 | | 40 |
| `2026-08-14-separate-artifact-kinds` | 2026-08-13 | | 40 |
| `2026-08-31-sync-a-change-into-the-accepted-contract` | 2026-08-31 | | 40 |
| `2026-09-02-carry-task-progress-into-the-listing` | 2026-08-31 | | 40 |
| `2026-09-02-state-a-changes-place-in-a-sequence` | 2026-09-02 | | 40 |
| `2026-09-06-distinguish-a-requirement-from-an-example` | 2026-08-24 | | 40 |
| `2026-09-11-migrate-an-openspec-project` | 2026-08-24 | | 40 |
| `2026-09-15-catch-a-templates-placeholder-prose` | 2026-09-11 | | 40 |
| `2026-09-18-establish-a-runbook` | 2026-09-16 | | 40 |
| `2026-09-26-carry-a-runbook-to-another-machine` | 2026-09-17 | `log-a-runbook-run` | 67 |
| `2026-09-26-log-a-runbook-run` | 2026-09-17 | `establish-a-runbook` | 69 |
| `2026-09-26-record-a-renamed-requirement` | 2026-09-26 | | 40 |
| `2026-09-28-confine-the-asset-check-to-the-project` | 2026-09-26 | | 40 |
| `2026-09-29-keep-evidence-out-of-document-discovery` | 2026-09-29 | `migrate-an-openspec-project` | 77 |

Every file sets `schema: spec-driven` and `created:`, and nothing else except the three `follows:`.

The 9 archives with no `.openspec.yaml`: `2026-08-17-project-bootstrap-cli`,
`2026-08-18-conform-generated-frontmatter-names`, `2026-08-18-install-the-full-workflow-set`,
`2026-08-21-check-release-hygiene`, `2026-08-21-distinguish-command-names-from-skill-names`,
`2026-08-22-accept-a-described-change`, `2026-08-24-pass-the-description-through`,
`2026-08-24-report-in-the-contributors-terms`, `2026-08-30-finish-the-archival`.

**15 files with `created:`, 3 with `follows:`, 9 archives with none.** This reproduces the expected
counts. No archive holds a `FOLLOWS` or `CREATED` file. All 24 hold `VERSION`.

## 2. What `new change` writes today

A scratch project from `htmlspec init --tools none --root …/scratchpad/record-when-a-change-was-created/proj`:

```
$ htmlspec new change probe
…/proj/htmlspec/changes/probe/proposal.html: created a proposal
…/proj/htmlspec/changes/probe/design.html: created a design
…/proj/htmlspec/changes/probe/tasks.html: created a tasks
…/proj/htmlspec/changes/probe: created a change with 3 document(s). The specification is produced at archival, from the completed change.
exit 0

$ find htmlspec/changes -print | sort
htmlspec/changes
htmlspec/changes/archive
htmlspec/changes/probe
htmlspec/changes/probe/design.html
htmlspec/changes/probe/proposal.html
htmlspec/changes/probe/tasks.html
```

**Three documents and nothing else.** No `CREATED`, no `FOLLOWS`, and no `VERSION`. With `--json`,
standard output is empty (0 bytes) and the same four lines go to standard error, so `new change`
has no structured payload.

What the program does write into a change directory, read from `crates/htmlspec-cli/src/commands.rs`
rather than measured: archival's `perform` writes `runbooks.json` and `VERSION` into the change
directory, inside a journaled transaction, immediately before it renames the directory into the
archive. Its comment calls an active change "a place this program never writes them", and finding
either one there is refused as an interrupted archival. So no non-document file the program writes
stays in an active change today.

`list` and `status` before the change, after `echo probe > htmlspec/changes/probe2/FOLLOWS`:

```
$ htmlspec list
probe   0/2  in-progress
probe2  0/2  in-progress  follows probe (active)
probe3  0/2  in-progress

$ htmlspec status --change probe2
  present proposal.html
  present design.html
  present tasks.html
  absent  specification.html
  1 of 2 task(s) done or dropped

$ htmlspec --json status --change probe2
{ "artifacts": [ … four entries … ], "change": "probe2",
  "tasks": { "open": 1, "openTasks": [ "task-example" ], "total": 2 } }
```

Neither reports anything about when a change was created, and `status` does not report `FOLLOWS`.

**`status` on an archived change.** A real archival of `probe3` was refused, as it should be,
because its specification still carried the template's example content. So the directory was
moved by hand to `htmlspec/changes/archive/2026-09-30-probe3`:

```
$ htmlspec status --change htmlspec/changes/archive/2026-09-30-probe3
  present proposal.html
  … 
  1 of 2 task(s) done or dropped
exit 0

$ htmlspec status --change 2026-09-30-probe3
htmlspec: no change named 2026-09-30-probe3 in …/proj/htmlspec/changes, and no directory at 2026-09-30-probe3
exit 2
```

`status` reaches an archived change by its directory, and not by its name.

## 3. The migration's decision 1

From `~/projects/html-spec-migration/make_decisions.py`, the decision as it was put in review
(lines 82-95):

> **Creation dates.** Fifteen changes record in `.openspec.yaml` the date the change was started,
> for example `migrate-an-openspec-project`, created 2026-08-24 and archived 2026-09-11. The
> destination has no field for a creation date. The 2026-09-09 plan dropped them, saying the
> archive directory's name already records the date. That was wrong: the name records the archive
> date. The plan proposed dropping the files and recording each date in its task. The alternative
> was to carry each file byte-identical into its change's `evidence/`, which stretches
> artifact-kinds' definition of evidence, "a record of why the change has the shape it does", to
> cover a configuration file.
>
> Decided: **Carry each one.** Every `.openspec.yaml` crosses byte-identical to
> `evidence/.openspec.yaml` in its change. The three that also name a predecessor are stated again
> as a `FOLLOWS` file, which is the declaration the program reads. The name inside `evidence/` is
> the migration's choice, since the contract is silent on it, and it keeps the source's.

Its rationale, written into each of the 15 entities (lines 50-54):

> Decided in review: each change's .openspec.yaml is carried byte-identical into that change's
> evidence, so the date the change was started survives as a record rather than only in this plan.
> The destination has no field for a start date. artifact-kinds' "A change's evidence is kept in
> the change and is not one of its documents" is what keeps it there unchecked and unrewritten. The
> contract is silent on the name, so the source's is kept.

From `~/projects/html-spec-migration/brief-draft.md`, the brief its drafters work from:

> 30: never touch a carried file (`VERSION`, `runbooks.json`, `FOLLOWS`, anything under
> 31: `evidence/`). Carried files are byte-identical records and were verified by digest.

> 91: 1. **Creation dates.** Each change's `.openspec.yaml` was carried byte-identical to
> 92: `evidence/.openspec.yaml` in its change. Where it named a predecessor, a `FOLLOWS` file was also
> 93: written. Both are already in place. You do not write either.

> 372: `evidence/.openspec.yaml` is the change's creation record and gets none.

**The stale rules-table row** (`make_decisions.py` lines 225-227), which still says the file is
folded or dropped:

```python
    ("<code>.openspec.yaml</code>", "folded or dropped",
     "a <code>FOLLOWS</code> file, or none",
     "artifact-kinds' \"A change states the change it follows\"."),
```

It contradicts decision 1, which carries every file. It is not only in the script. The generated
`decisions.json` still carries the phrase "a `FOLLOWS` file, or none". The staged tree already
holds what decision 1 describes:

```
$ grep -c 'evidence/.openspec.yaml' decisions.json
16
$ find staged -path '*evidence/.openspec.yaml' | wc -l
15
$ find staged -name FOLLOWS
staged/htmlspec/changes/archive/2026-09-26-carry-a-runbook-to-another-machine/FOLLOWS
staged/htmlspec/changes/archive/2026-09-26-log-a-runbook-run/FOLLOWS
staged/htmlspec/changes/archive/2026-09-29-keep-evidence-out-of-document-discovery/FOLLOWS
```

So the migration already preserves all 15 dates, without any field to put them in.

## 4. Where else a start date could be read

The archive directory is named for the day the change was archived. The other candidate is
version control, so for each of the 15 changes I took the first commit touching its active or
archived path (`git log --reverse --format=%ad --date=short -- openspec/changes/<name>
openspec/changes/archive/<dir>`):

| Archive | `created:` | First commit | Archived | Days open | First commit vs `created:` |
| --- | --- | --- | --- | --- | --- |
| `2026-08-10-html-spec` | 2026-08-04 | 2026-08-07 | 2026-08-10 | 6 | +3 |
| `2026-08-13-content-address-runtime-assets` | 2026-08-10 | 2026-08-12 | 2026-08-13 | 3 | +2 |
| `2026-08-14-separate-artifact-kinds` | 2026-08-13 | 2026-08-13 | 2026-08-14 | 1 | same day |
| `2026-08-31-sync-a-change-into-the-accepted-contract` | 2026-08-31 | 2026-08-31 | 2026-08-31 | 0 | same day |
| `2026-09-02-carry-task-progress-into-the-listing` | 2026-08-31 | 2026-08-31 | 2026-09-02 | 2 | same day |
| `2026-09-02-state-a-changes-place-in-a-sequence` | 2026-09-02 | 2026-09-02 | 2026-09-02 | 0 | same day |
| `2026-09-06-distinguish-a-requirement-from-an-example` | 2026-08-24 | 2026-08-31 | 2026-09-06 | 13 | +7 |
| `2026-09-11-migrate-an-openspec-project` | 2026-08-24 | 2026-08-31 | 2026-09-11 | 18 | +7 |
| `2026-09-15-catch-a-templates-placeholder-prose` | 2026-09-11 | 2026-09-13 | 2026-09-15 | 4 | +2 |
| `2026-09-18-establish-a-runbook` | 2026-09-16 | 2026-09-17 | 2026-09-18 | 2 | +1 |
| `2026-09-26-carry-a-runbook-to-another-machine` | 2026-09-17 | 2026-09-17 | 2026-09-26 | 9 | same day |
| `2026-09-26-log-a-runbook-run` | 2026-09-17 | 2026-09-17 | 2026-09-26 | 9 | same day |
| `2026-09-26-record-a-renamed-requirement` | 2026-09-26 | 2026-09-27 | 2026-09-26 | 0 | +1 |
| `2026-09-28-confine-the-asset-check-to-the-project` | 2026-09-26 | 2026-09-28 | 2026-09-28 | 2 | +2 |
| `2026-09-29-keep-evidence-out-of-document-discovery` | 2026-09-29 | 2026-09-29 | 2026-09-29 | 0 | same day |

- **The first commit falls on a different day from `created:` in 8 of 15**, by up to 7 days. It is
  never earlier. Version control records when work was committed, not when it began.
- `created:` is on or before the archive date in 15 of 15. Spans run from 0 to 18 days.
- The eleven active changes in this repository were all created on 2026-09-30 and are untracked, so
  none of them has a first commit yet.

## 5. OpenSpec: which clock it uses, and what reads `created:`

From the installed 1.13.1 (`/nix/store/…-openspec-1.13.1/lib/openspec/dist`):

```js
// utils/date.js
/**
 * Formats a date using the effective local time zone of the Node.js process.
 */
export function formatLocalDate(date = new Date()) { … getFullYear() … getMonth() … getDate() … }

// utils/change-utils.js:144, writing .openspec.yaml
        created: formatLocalDate(),
// core/archive.js:1142, naming the archive directory
            : `${formatLocalDate()}-${changeName}`;
```

**OpenSpec writes both of its dates from one local-time clock.** In a scratch directory:

```
$ openspec new change probe
…
Next: openspec status --change probe
$ find openspec -type f | sort
openspec/changes/probe/.openspec.yaml
openspec/config.yaml
$ cat openspec/changes/probe/.openspec.yaml
schema: spec-driven
created: 2026-09-30

$ TZ=Pacific/Kiritimati openspec new change probe-kiritimati
created: 2026-10-01   (UTC date at that moment: 2026-09-30)
```

**Nothing in OpenSpec reads `created:` beyond checking its form.** `grep` across `dist/` finds it in
`core/change-metadata/schema.js`, which validates it against `/^\d{4}-\d{2}-\d{2}$/`, and in the
writer above. No command's output carries it. `openspec list --json` reports each change's
`lastModified`, which it computes from file times, and `openspec status --change … --json` carries
no date. The only other mention is in an agent instruction,
`core/templates/workflows/bulk-archive-change.js`, for two changes whose deltas touch one capability:

> If both implemented -> apply in chronological order (older first, newer overwrites)

with an example that reads "Checking add-rest-api (created 2026-01-10)" and "Checking add-graphql
(created 2026-01-15)".

**A malformed `created:`** (`created: yesterday`):

```
$ openspec status --change probe
✖ Error: Invalid metadata: [
  { "origin": "string", "code": "invalid_format", "format": "regex",
    "pattern": "/^\\d{4}-\\d{2}-\\d{2}$/", "path": [ "created" ],
    "message": "created must be YYYY-MM-DD format" } ]
exit 1
$ openspec list
Changes:
  probe                No tasks      just now
  …
```

`status` fails on the whole change, and `list` lists it anyway.

## 6. html-spec's clock

```
$ grep -rn -E 'SystemTime|UNIX_EPOCH' crates conformance
crates/htmlspec-cli/src/commands.rs:4033:    let seconds = std::time::SystemTime::now()
crates/htmlspec-cli/src/commands.rs:4034:        .duration_since(std::time::UNIX_EPOCH)
$ grep -rn -i -E 'chrono|jiff|"time"' Cargo.toml crates/*/Cargo.toml
(nothing)
```

`today()` is documented "Today, as `YYYY-MM-DD` in UTC". It divides seconds since the epoch by
86,400 and converts the day with a hand-written `civil_from_days`. A clock before the epoch falls
through `.unwrap_or_default()` to zero seconds, which is 1970-01-01. Its one caller is archival's
`perform` (line 3762), which names the archive directory `archived_name(&name, &today())`.
`crates/htmlspec-cli/src/runs.rs` reads no clock, and a run's record carries no date.

This machine is on CDT (UTC−5). At 16:53 CDT on 2026-09-30 the local and UTC dates agree. From 19:00
to midnight CDT they do not: an archival run then names its directory for the next day.

## 7. What earlier records said about these dates

The 2026-09-07 records in `2026-09-11-migrate-an-openspec-project` dropped `created:` and described
its values:

> **In all 8, `created:` disagrees with the directory name.** Every one reads `created: 2026-08-24`,
> including `2026-09-06-distinguish-a-requirement-from-an-example` and `2026-08-10-html-spec`. The
> field is unmaintained — copied forward rather than written per change — so it is not a fact worth
> carrying, and carrying it would import 8 wrong dates.
> (`evidence/the-mapping-table-and-its-audit-2026-09-07.md`, lines 184-187)

The design says the same at lines 364-367 ("every one reading `created: 2026-08-24`"), and
`tasks.md` line 173 says instead that "every one reads `created: 2026-08-04`".

**Neither is what the files held.** By exact path (not `git log --follow`), each of the 15 files
has had one content since it was first committed. The eight present on 2026-09-07 held six
different dates: 2026-08-04, 08-10, 08-13, 08-31 (twice), 09-02, and 08-24 (twice). They
"disagree" with their directory names because one is the day work began and the other the day it
was archived. The 2026-09-29 review corrected the second half of this ("That was wrong: the name
records the archive date"), and section 3 quotes it.

A trap for anyone repeating this: `git log --follow` on these files shows apparent edits such as
`-created: 2026-08-04` / `+created: 2026-08-10`. Git pairs identical 40-byte files across changes
as renames. By exact path, none was ever edited.

Three answers recorded in `2026-09-06-distinguish-a-requirement-from-an-example/evidence/transcripts/`
reached the "no reader" reading from the contract:

> The creation date has no reader: `artifact-kinds`' *A change states the change it follows* forbids
> the program inferring an ordering from a change's creation date, … (`after-answer.md`, line 23)

> The creation date has no consumer either: `artifact-kinds` forbids inferring a change's ordering
> from its creation date, so carrying the field would create a value nothing reads.
> (`before-answer.md`, line 14)

> For an archived change the date is not lost — it is already in the archive directory's name.
> (`control-answer.md`, line 14)

The third is the reading the 2026-09-29 review found wrong.
