# The namespace decides the severity, so `kind/` could not carry this rule

Section 3, recorded 2026-09-04 against `main` at `80558b5` plus the working tree.

Task 3.2 said to add the diagnostic under `kind/`. Task 3.3 said to report it as a warning.
**Those two instructions cannot both hold**, and the task list is amended rather than one of them
followed silently. This records why, and what the amendment cost.

## The conflict, measured

Severity is a property of the namespace. `crates/htmlspec-validate/src/diagnostic.rs`:

```rust
pub const fn default_severity(self) -> Severity {
    match self {
        Self::Review => Severity::Warning,
        _ => Severity::Error,
    }
}
```

There is no per-rule override anywhere in the crate, and the invariant is enforced rather than
intended — `a_namespace_decides_severity_rather_than_each_rule` asserts that every namespace
except `review/` defaults to an error. The module comment states the arrangement as the design:

> A rule identifier is namespaced by concern, and the namespace carries the severity policy
> rather than each rule restating it… That is what makes "can this be a warning?" a question
> about the namespace instead of a judgement call per rule.

## The second consequence, which decided it

A `kind/` rule downgraded to a warning is still **promotable**. `is_promotable` returns false for
`review/` alone, so `--strict` restores the error everywhere else — and `scripts/check` runs
strict. A downgrade would therefore fail a freshly created document under the project's own gate,
which is the exact outcome task 3.3 exists to prevent.

So `kind/` breaks the change's stated requirement twice, not once. That is what makes this an
amendment rather than a preference.

## The precedent was already in the file

`review/unverified-requirement` is a rule about what a declared kind requires of its content,
which is `kind/`'s definition, and it lives in `review/` anyway. Its comment reads as though
written for this case:

> In `review/*` rather than `kind/*`, and the placement is the decision rather than a filing
> choice. Verification arrives as work completes, so a specification in progress legitimately has
> requirements not yet verified. This namespace is the one strict mode refuses to promote, so the
> report stays advisory under every strictness — in `kind/*` it would be promotable, and
> `--strict` would fail a specification for being in progress, gating the work that produces the
> verification.

Substitute example content for verification and every clause holds. **The test is temporal.** Ask
when the reported condition resolves. A misplaced task is wrong the moment it appears and wrong
until someone moves it, so it is an error. Example content is normal at creation and wrong at
archival, and the work that clears it is the work a gate would block.

## The severity split, proven at both operations

The contract requires that validation and archival differ. One document, two operations:

```console
$ htmlspec validate htmlspec/changes/keeps-example/specification.html
  0 errors, 9 warnings: the document is accepted
$ echo $?
0

$ htmlspec archive keeps-example --check
  0 errors, 9 warnings: the document is accepted
htmlspec: archival does not proceed while the specification carries a template's example
content: req-example, scenario-example, decision-example, evidence-example, risk-example,
verify-example, figure-example. An example requirement states no obligation, so archiving one
would put an obligation nobody wrote into the project's accepted specifications. Replace each
of them with the change's own, and run this again
$ echo $?
1
```

Accepted by one, refused by the other. Replacing the identifiers clears the refusal, which is
what makes the check about the identifier rather than the document's shape.

Every kind `htmlspec new document` writes is accepted, under default strictness **and** under
`--strict`:

| Kind | `validate` | `validate --strict` |
| --- | --- | --- |
| `proposal` | accepted, no diagnostics | accepted |
| `design` | accepted, no diagnostics | accepted |
| `tasks` | accepted, 2 warnings | accepted |
| `specification` | accepted, 9 warnings | accepted |

## One thing the rule needed that the tasks did not name

`figure-example` is one of the eight identifiers, and a figure is **not** a semantic unit —
`Model::units()` does not yield one, because the model carries figures in their own collection.
Reading only the units left one of the eight uncovered while the derivation reported it as in
scope. The rule checks figures beside the units, and
`a_figure_carrying_an_example_identifier_is_reported` pins it.

Found by running the rule against a real template rather than by reading the model, which is the
argument for task 1.6 having demonstrated the hazard rather than reasoned about it.

## Task 3.7: it does not over-report

Measured against every fixture in the repository, before and after, with the binary rebuilt for
each:

| | Errors | Warnings |
| --- | --- | --- |
| `main` at `80558b5` | 110 | 122 |
| With this rule | 110 | 459 |

**Zero new errors.** The 337 added warnings fall in the 49 fixture files that genuinely contain a
template's example identifier, which `grep -rl 'id="[a-z]*-example"' fixtures/` confirms is
exactly 49 files. The conformance suite passes: its fixtures assert a verdict and the rules that
must appear rather than an exhaustive list, so an added advisory does not break one.

This repository's own HTML documents report nothing — `README.html` and the two design documents
carry no example identifier.

`scripts/check`: all 9 stages pass.
