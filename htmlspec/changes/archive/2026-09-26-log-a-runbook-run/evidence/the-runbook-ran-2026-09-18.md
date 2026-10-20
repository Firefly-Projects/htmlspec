# The release runbook's second real run, and whether it worked as written

The predecessor's stated success test is that a change's release section becomes **one item saying to
run the runbook**. This change's `tasks.md` §10 is three items rather than one, and the three are the
run, the increment decision, and the check — so the test is not quite met and the reason is recorded
below. The procedure itself worked as written, with one known defect confirmed as worse than stated
and one new observation.

## Worked as written

**The step order is right and both hazards arrived exactly as the runbook's requirements state
them.** That is the second independent confirmation, which is what the runbook exists for.

`req-version-before-regenerate` says a run MUST advance the version before regenerating, and MUST NOT
treat the regeneration as conditional on whether any workflow body changed. Checked at both points
rather than inferred, which is what the requirement's own verification method asks for:

```
$ # version raised 0.13.0 → 0.14.0, cargo update --workspace --offline: 7 crates, nothing else
$ scripts/check-installed
the installed instruction files are not what this binary writes.
```

**Nothing but the version had changed**, so the version alone is the trigger. After regenerating, the
diff across the installed files reduces to exactly two distinct lines:

```
$ git diff -- .claude .codex .pi | grep '^[+-]' | grep -v '^[+-][+-]' | sort -u
-  generatedBy: htmlspec 0.13.0
-generatedBy: htmlspec 0.13.0
+  generatedBy: htmlspec 0.14.0
+generatedBy: htmlspec 0.14.0
```

45 files, one line each, nothing else — read from the whole diff rather than sampled. This change
touched no workflow body at all, which makes it the cleanest possible instance of the hazard: the
regeneration was *entirely* the version stamp.

`req-init-bootstraps-a-stray-project` says a run MUST remove the `htmlspec/` and `spec.nix` the
bootstrap creates, and MUST run the suite afterwards rather than assuming the removal was complete.
Both appeared, and `htmlspec/` held all four places including `runbooks/` — the predecessor's task 5.2
visible in the real command's output. Removed; `check-installed` then reports 45 files matching and
`htmlspec assets .` exits 0 with `each stored asset appears once`.

## The known defect, confirmed and worse than the runbook states

The regeneration step says to run `htmlspec init`. The instruction to this change warned that this
resolves to the Nix profile binary at the *previous* version. Measured:

```
$ which htmlspec                 ~/.nix-profile/bin/htmlspec
$ htmlspec --version             htmlspec 0.12.0 (unknown)
$ ./target/debug/htmlspec --version   htmlspec 0.14.0 (48ffe3b8b87d-dirty)
```

**The profile binary is at `0.12.0`, which is two releases behind rather than one.** So the defect is
not "the stamp would read the previous version" — it is that the stamp would read a version from two
releases ago, and `check-installed` would fail in a way that looks like the regeneration did not
happen rather than like the wrong binary ran.

The runbook's own prose is what makes this a defect rather than an ambiguity: it says *"Regenerate
through the command rather than editing the stamp — a hand-edited stamp is a file claiming a run that
never happened"*, which is right, and then names a command that claims a run by a binary that is not
the one being released. **The fix is a sentence in the runbook** naming the freshly built binary, and
it is left for the change that edits the runbook because editing it here would mean shipping a runbook
edit inside a change about run records. Recorded rather than silently worked around, per the
instruction.

## A new observation: the runbook cannot record its own run

**This change's release is the first one that *could* have produced a run record, and it did not —
because this repository's runbook is at `runbooks/release/` rather than `htmlspec/runbooks/release/`.**

`htmlspec run` resolves a runbook through `ProjectRoot::runbook`, which composes under
`htmlspec/runbooks/`. This repository is deliberately not an html-spec project, for the measured
reason in `the-location-is-blocked-2026-09-17.md`: creating `htmlspec/` at its root makes
`self-assets` red by comparing across the four stores its conformance fixtures deliberately carry.

So the operation this change ships cannot be pointed at the runbook this change ran. That is not a
defect in either — it is the predecessor's location finding arriving at a second consequence, and it
is why the design states plainly that the rule is dogfooded in a scratch project and pinned by
fixtures rather than by this repository's suite. What closes it is migrating this repository to
`htmlspec/`, which is `migrate-an-openspec-project`'s unfinished half.

**The run was recorded in a scratch project instead**, which is the weaker form and is what is
available:

```
$ htmlspec run release --as 0.14.0
/private/tmp/rb-runs/htmlspec/runbooks/release/runs/0.14.0.html: started a run of release
  4 step(s) to work, 5 requirement(s) to record
$ htmlspec validate  …/runs/0.14.0.html     no diagnostics: the document is accepted
$ htmlspec format --check …/runs/0.14.0.html   is canonical
```

Four steps copied with their inline markup intact, five requirements copied, five not-yet-run entries.

## Whether the success test was met

**Not quite, and the gap is honest rather than a failure of the runbook.** The predecessor's test is
that a release section becomes one item saying to run the runbook. This change's §10 has three:

1. run the runbook — which *is* the one item, and it is the whole of the procedure;
2. choose the increment and state the reasoning;
3. confirm `scripts/check-release` passes.

Item 3 is redundant with the runbook's own fourth task and could be dropped. **Item 2 cannot**, and
that is the finding: `archive.md` reserves the increment decision for a human and requires the run to
*state its reasoning*, so the reasoning has to be written somewhere a reviewer reads. The runbook says
the decision is yours; it cannot hold the answer, because the answer is different every release.

**That is exactly what a run record is for**, and it is where the reasoning would go once this
repository can hold one — a `method` paragraph on a verification of
`req-the-increment-is-judgment`. So the success test is met by the runbook and blocked by the location,
and the item that cannot collapse is the one the mechanism this change ships would absorb.

## The increment: minor, and why

**`0.13.0` → `0.14.0`.** A new capability (`runbook-execution`), a modified one, a new CLI subcommand
with two modes, a new diagnostic identifier, and two new conformance fixtures — additive throughout,
with no schema change, no new artifact kind, no new unit, and no existing document changing meaning or
validity. Every existing command prints exactly what it printed.

The same shape as `0.10.0`, `0.11.0`, and `0.13.0`, each of which added a capability. `0.6.1` is still
the only patch release this project has made, and it changed no crate. Not a major: nothing is removed
or redefined, and `rules::ALL` is untouched so no project's validation output changes.

**`archive.md` reserves this decision for a human, so it is flagged for confirmation rather than
presented as settled.**
