# The measurement repeated against the built change

**Date:** 2026-10-06
**Tasks:** 8.1 and 8.2
**Binary:** `htmlspec 0.22.0 (940e37a8bbe8-dirty)`, built from this change's branch. The dirt is the
installed instruction copies task 7.3 regenerated and this change's task list. Every `htmlspec` below
is `./target/debug/htmlspec`.
**Projection:** `project-model`, rebuilt against this branch's `crates/htmlspec-parse` before these
runs. The copy built for task 1.2 linked the parser as it was then, and still kept a deleted word in
the statement; task 7.1 records how that was found.

The same script as task 1.2, appendix A of `the-measurement-2026-09-30.md` with the `ins-del`
variant, in a fresh scratch project built with `init`, so the specification names the new stylesheet
revision.

## Task 8.1: sections 2, 3, and 5

### The projection

```
{"id":"req-a-single-command-exposes-validate-format-and-init","change":"modified","statements":[… 2 statements …, "… list, status, sync, instructions, and store name the operations OpenSpec also provides. format and assets name operations it does not."],"deletions":null}
{"id":"req-a-project-has-a-declared-root-holding-one-asset-store","change":"modified","statements":["An HTMLSpec project SHALL be identified by a directory containing an htmlspec/ directory. That directory SHALL hold the project's accepted specifications, its active changes, its archive, and its runbooks, each in its own place."],"deletions":["and"]}
```

**The runbook statement reads "… its active changes, its archive, and its runbooks, each in its own
place.", and its `deletions` holds "and".** On 2026-09-30 and in task 1.2 it read "its active
changes, and its archive, and its runbooks". The inserted `sync` is in its statement, and that
requirement records no deletion. As task 8.1 expected.

### Validation

```
$ htmlspec validate htmlspec/changes/measure-marked-words/specification.html
  warning: review/stale-reference at verify-the-spellings: …
  warning: review/stale-reference at verify-the-places: …
  0 errors, 2 warnings: the document is accepted
validate exit 0
{"accepted":true,"errors":0,"warnings":2,"rules":["review/stale-reference","review/stale-reference"]}
```

**No `schema/change-markup`**, where task 1.2 reported three. The two warnings are the ones the
measurement has always reported, for verifications naming a requirement the change modifies. The
same bytes copied to `htmlspec/specs/cap/spec.html` report three `schema/change-markup` lines and
exit 1, one for each element, as task 8.1 expected.

### The formatter

As measured on 2026-09-30 and in task 1.2: the inline form is canonical as written, the block-level
`<del><p>` and `<ins><p>` are laid out with each paragraph on its own line, and a second run changes
nothing. What changed is the model of the block-level copy. Before and after formatting alike, it
now holds three statements and `deletions: ["No other spelling SHALL name a command."]`, where task
1.2 found four statements, the deleted paragraph among them.

### The stylesheets

`assets/sha256-ebf35a36fe4dc1a7/htmlspec.css: 12` lines name `ins` or `del`. Every other revision
still names neither.

## Task 8.2: the staged migration against the baseline

`project-model` over the 116 `.html` files under the staged `htmlspec/`, compared with the outputs
task 1.4 kept: `diff -r` exits 0, so **every model is byte for byte the baseline.**

`htmlspec --json validate` from the staged project's root:

```
{"documents":109,"accepted":true,"errors":0,"warnings":781}
317 warning review/stale-reference
193 warning review/stale-verification
271 warning review/unverified-requirement
```

**The baseline's counts, rule for rule.** No staged document carries change markup, so the
projection change and the relaxed rule reach none of them. It took 19.78 seconds of CPU time,
against 20.45 for the baseline.
