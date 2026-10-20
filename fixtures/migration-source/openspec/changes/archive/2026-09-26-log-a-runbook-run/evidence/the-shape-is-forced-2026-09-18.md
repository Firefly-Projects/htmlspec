# The run record's shape, re-measured rather than inherited

The proposal states four constraints as measured facts and says the design rests on them. **The
instruction was to confirm they still hold rather than to trust the claim**, so every one below was
re-run on the binary at `056df05` (version `0.13.0`) with no modification, in a scratch project at
`/tmp/rb-runs` holding a copy of this repository's release runbook at
`htmlspec/runbooks/release/`.

## Setup

```
$ htmlspec init --tools none                       # /private/tmp/rb-runs, version 0.0.0
$ cp runbooks/release/{specification,tasks}.html htmlspec/runbooks/release/
$ htmlspec list --runbooks
release  specification, tasks
$ htmlspec validate htmlspec/runbooks/release
2 document(s): 0 error(s), 0 warning(s)
```

## 1. A reference resolves within one document only — **confirmed, and it is an error**

A `tasks` document at `htmlspec/runbooks/release/runs/probe.html` whose verification names
`req-version-before-regenerate`, which lives in the runbook's `specification.html` one directory up:

```
error: refs/unresolved at verify-version-before-regenerate: this verification references
  req-version-before-regenerate, and no unit in the document declares that identifier. A
  reference that resolves to nothing records a relationship to a thing that does not exist
1 error, 0 warnings: validation failed                                            exit 1
```

**So the copying is forced, not chosen.** A run record that pointed outward at the runbook would be
invalid on the day it was written. The proposal's claim holds exactly as stated, including that the
severity is an error rather than a warning.

## 2. `kind=specification` fails on every step — **confirmed, and the count is per task**

The same body, declared two ways. Byte-identical content apart from the `htmlspec-kind` meta and
the masthead's kind chip:

| Kind | Result |
| --- | --- |
| `tasks` | `no diagnostics: the document is accepted`, under default **and** `--strict` |
| `specification` | `2 errors`: `kind/task-in-specification` at `task-advance-the-version` and at `task-regenerate-the-instructions` |

The record carried two task units and drew two errors, which is the "on every step" the proposal
claims — one diagnostic per task rather than one for the document. A real release run copies four
steps and would draw four.

**And the accepting direction was measured too**, which matters more than the refusal: a `tasks`
document carrying task units, requirement units with scenarios, and verification units together is
accepted with **zero** diagnostics. The tasks rule is completeness (`kind/no-tasks` wants at least
one task) rather than placement, so nothing objects to the requirements sitting beside the tasks.

## 3. The three further constraints — **all three confirmed, each with its own diagnostic**

Each produced by one edit to the accepted record above, so each isolates its own rule:

| Edit | Reported |
| --- | --- |
| the copied requirement's `scenario` removed | `error: schema/invalid at req-version-before-regenerate (/requirements/0/scenarios): [] has less than 1 item` |
| `MUST` weakened to `should` | `error: schema/weak-modal at req-version-before-regenerate: no statement of this requirement uses SHALL or MUST` |
| `data-change="unchanged"` → `"added"` | `warning: review/stale-verification at verify-version-before-regenerate: this verification records the outcome passed against req-version-before-regenerate, which this change marks added` |

Two errors and one warning, which is the split the proposal states. The `unchanged` requirement is
therefore the same finding `the-canonical-documents-change-state-2026-09-17.md` recorded for the
runbook's own documents, arriving in the run record for the same mechanical reason: a run record is
the *only* other document in a runbook whose units reference each other.

## 4. The gap the conformance rule exists for — **measured, and it is total silence**

The one case the proposal names as the rule's subject: a run record carrying a copied requirement
that **no verification names**.

```
$ htmlspec validate .../runs/missing-verification.html
  no diagnostics: the document is accepted
$ htmlspec validate --strict .../runs/missing-verification.html
  no diagnostics: the document is accepted
```

**Nothing is reported, at either strictness.** `review/unverified-requirement` is the rule whose
subject this is, and `requirements_are_verified` in `crates/htmlspec-validate/src/rules/review.rs`
returns early unless the document's kind is `specification`:

```rust
if schema_facts::artifact_kind("specific").as_deref() != Some(kind) {
    return;
}
```

That early return is correct for its own reason, stated in the rule's own doc comment — a
requirement unit in a proposal or a design is an author arguing for it rather than stating it. But
it means the kind the run record is **forced** into by measurement 2 is precisely the kind the
existing rule cannot see. The two forced facts point in opposite directions, and the project-level
rule is what closes the gap between them.

## 5. Asset references resolve at `runs/` depth with no new mechanism

```
$ htmlspec new document tasks htmlspec/runbooks/release/runs/probe.html --subject "Probe run"
htmlspec/runbooks/release/runs/probe.html: created a tasks
$ grep -o 'href="[^"]*htmlspec.css"' .../runs/probe.html
href="../../../assets/sha256-e4f5de7cecc4f691/htmlspec.css"
```

Three levels rather than two, correctly — `runbooks/<name>/runs/` is three below `htmlspec/`. The
store's location is derived from the project by `ProjectRoot::assets()`, so this needed nothing.
`asset/duplicate` has nothing to report and `htmlspec assets .` exits zero.

## 6. `validate` and `format` descend into `runs/`, and `list --runbooks` does not

Both true, and both are the behaviour wanted:

```
$ htmlspec validate htmlspec/runbooks            4 document(s): 0 error(s), 0 warning(s)
$ htmlspec format --check htmlspec/runbooks      4 documents, all canonical
$ htmlspec list --runbooks                       release  specification, tasks
```

`validate` walks every `.html` beneath its path, so a run record is checked by the ordinary path
with no work — which is what `runbook-authority` requires and what this repository's new
`self-runbooks` stage will do to any run record committed here. `list --runbooks` reads immediate
subdirectories and reports which of `RUNBOOK_DOCUMENTS` are present, so `runs/` is neither listed as
a runbook nor counted as a document. **A run record is invisible to the enumeration**, which is a
finding rather than a defect: it is the same reason the canonical procedure's fraction is not
printed, and where run progress is reported is a decision this change makes in its design.

## What this measurement did not find

The proposal's four claims all held, with no correction. That is worth stating plainly because the
predecessor's evidence files each corrected something in the proposal above them — the status gap
file corrected a claim about `status`, the change-state file corrected its own first draft. This one
corrects nothing. The claims were measured before they were written, and re-measuring found the same
numbers.
