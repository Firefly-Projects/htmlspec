# A runbook's units are `unchanged`, and the templates cannot know that

**A finding the design did not anticipate, hit while writing the release runbook rather than
predicted.** It is small, it has a clean answer, and the reason it is worth an evidence file is
that the answer is the same reasoning as "the canonical procedure's tasks stay `todo` forever"
applied to a different attribute.

## What happened

`htmlspec new document specification` writes every unit with `data-change="added"`, which is
correct for a change: a change's specification states what that change establishes, so its
requirements are being added to the contract.

Written that way, the release runbook's specification reported **seven warnings**:

```
review/unverified-requirement  ×2   (two requirements had no verification)
review/stale-reference         ×2   evidence references a requirement marked added
review/stale-verification      ×3   a verification records `passed` against a requirement
                                    this change marks added
```

The last five are the interesting ones. `review/stale-verification`'s message states the
problem exactly: *"this verification records the outcome `passed` against
`req-version-before-regenerate`, which this change marks `added`. Re-check it, or the record
attests to an obligation that has since moved."*

**The rule is right and the document was wrong.** Nothing is adding these requirements. The
hazards they state were measured at `0.10.0` and again at `0.12.0`, three weeks apart, before
this change existed. A verification recording `passed` against a requirement marked `added` is
a claim to have checked something that is not there yet, and that is what the warning says.

## The fix, and why it is the right one rather than the quiet one

`data-change="unchanged"` on every unit. Warnings went 7 → 2, and the remaining two were
genuine gaps — two requirements with no verification — which were closed by writing the two
verifications rather than by deleting the requirements.

**Final state: 0 errors, 0 warnings, on both documents, under default and strict strictness,
both canonically formatted.**

### Correction: "on every unit" was false when this file first said it

**This section originally claimed `unchanged` on every unit, and the shipped `tasks.html` carried
`added` on all four of its task units.** Only `specification.html` was fixed — 24 units, all
`unchanged` — because that is the document the seven warnings came from. The tasks document
reported nothing, so nothing sent me back to it, and I wrote the general claim from the specific
fix.

Found in review of PR #29, not by any check. Corrected: all four task units are now `unchanged`,
and both documents are `0 errors, 0 warnings` under both strictnesses.

**The reason the mistake was available is the finding itself, one level up.** `runbook-authority`
says *"a unit in a runbook's canonical documents SHALL declare that it is unchanged"* — documents,
plural, which covers `tasks.html`. But the diagnostics that taught me the rule all came from the
specification, because that is the only document whose units *reference each other*: a verification
naming a requirement marked `added` is what `review/stale-verification` fires on. A runbook's task
units reference nothing, so nothing fires, so the contract's plural had no measurement behind it in
the document where it was violated.

**Validation catches this in neither mode**, verified in place rather than on a copy — a copy
outside the project fails on asset resolution and confounds the test:

```
$ htmlspec validate runbooks/release/tasks.html          # with data-change="added"
  no diagnostics: the document is accepted
$ htmlspec validate --strict runbooks/release/tasks.html
  no diagnostics: the document is accepted
```

So this is a contract clause with no mechanical check, which is the shape `project-bootstrap`
names when it says a rule nothing checks is not a rule — and this change's own subject is that
shape. **A rule is warranted and is not written here**, on two grounds. The proposal forbids
touching `crates/htmlspec-validate/src/rules/` and that constraint still holds; and the rule
`runbook-authority` already forbids is exactly the one needed — *"no rule SHALL exist whose subject
is that a document belongs to a runbook."* A per-document rule cannot see a document's location,
so the honest form is a project-level check like the one `log-a-runbook-run` plans beside
`topology.rs`, whose subject is a project rather than a document. That is a successor change.

`unchanged` is the honest value and not merely the quiet one. A runbook is not a change. Its
canonical documents state a procedure that exists and hazards that were already measured, so
every unit in them describes something settled. The change state describes a unit's relationship
to a change in progress, and a runbook has none — which is the same reason its tasks stay `todo`
forever, arriving at a different attribute.

## Why this is a finding rather than a bug

Nothing is wrong with the templates. `htmlspec new document` cannot know whether the document
it is writing belongs to a change or to a runbook — that is what a workflow would know, and
this change deliberately has no workflow. So an author writing a runbook by hand meets seven
warnings whose cause is one attribute, and nothing tells them which.

**Three ways to close it, none of them this change's:**

1. **A workflow for authoring a runbook**, which would write `unchanged` because it knows what
   it is writing. Deferred as `author-a-runbook-through-the-change-lifecycle`, and this change
   is required to be provable without one.
2. **`unchanged` stated in `runbook-authority`**, so an author reading the contract finds it.
   **Done** — the requirement that the canonical procedure carries no completion state is where
   the general form belongs, and it now covers the change state as the same fact about the same
   thing.
3. **A rule reading a document's location.** Rejected. `runbook-authority` states that no rule
   shall exist whose subject is that a document belongs to a runbook, and this would be exactly
   that rule.

## What this measured about the warnings themselves

Worth stating because it cuts the other way from the finding: **`review/stale-verification` and
`review/stale-reference` did useful work here on their first contact with a document neither
was designed for.** Both fired on a real defect in a document nobody had written before, and
the messages named the attribute to change. Neither gates, so the document was accepted the
whole time — which is the `review/*` policy working as stated rather than in spite of it.
