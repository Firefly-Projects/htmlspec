# Driving the lifecycle through the installed workflows

Task 10.4. A body is prose and prose has no compiler, so using the instructions is the only
check that exists. Run in `/tmp/g10`, the same scratch project as the install measurement.

## Every invocation the seven bodies name

Extracted from the seven bodies, then each one run:

| Invocation | Named in | Result |
| --- | --- | --- |
| `htmlspec new change` | `new`, `ff` | Writes proposal, design, tasks. Reports the resolved path. |
| `htmlspec new document` | `continue` | Writes one document of a kind into the change. |
| `htmlspec new document specification` | `archive` | Writes a valid specification. |
| `htmlspec validate` | `new`, `continue`, `ff`, `apply`, `verify` | Whole project, from any directory. |
| `htmlspec format` | `apply` | Reported all four documents canonical. |
| `htmlspec archive` | `archive` | Hits the version gate, then validates. |
| `htmlspec archive add-dark-mode` | `archive` | The named-change form resolves. |

`explore` names no invocation, which is deliberate and is the change's argument in its
strongest form: a workflow with no command is legitimate, so the hazard was never "a
workflow without a command".

## The walkthrough

```
htmlspec new change add-a-greeting --subject "Add a greeting"
  → proposal.html, design.html, tasks.html; 3 documents
htmlspec validate
  → 3 documents: 0 errors, 0 warnings
htmlspec validate htmlspec/changes/add-a-greeting
  → the change-directory form works, as `continue` step 1 says
htmlspec new document specification .../specification.html --subject "Add a greeting"
  → created a specification
htmlspec format
  → all four documents canonical
htmlspec archive add-a-greeting
  → refused: "the declared version is still 0.0.0, which is the last archived version"
(raise version to 0.1.0 in spec.nix)
htmlspec archive add-a-greeting
  → 0 errors, 2 warnings; "the specification validates; archival may proceed"
```

Two findings worth recording, neither a defect:

**The version gate fires exactly as `archive.md` step 3 describes it**, including the part
about the division of labor: the refusal says "Advancing it is your work — this command
compares and never edits". The body told the agent to expect that, and it did.

**The two warnings are `review/stale-reference`, on the template's own example units.** The
archival body says warnings of that kind usually mean a unit's change state needs settling
now that the work is done, and that is what these are: the template ships an evidence unit
and a verification unit referencing a requirement marked `added`. An agent following the
body would reconcile them while replacing the example content. The gate does not block, which
`archive.md` also states.

**`htmlspec validate htmlspec/specs` reports "no documents found"** on a project that has
archived nothing. That is correct rather than an error, and `archive.md` step 5 names the
command in a context where the accepted specifications already exist. No change needed.

## What reading the bodies as an agent surfaced

One real defect, in output rather than in a body — see the install evidence for the
`ask your assistant to new` ungrammaticality and its fix.

Nothing in the six new bodies named a command that does not exist, which is what
`no_shipped_body_names_an_absent_command` already held before this run. The value of the run
is the part that test cannot reach: whether the commands, run in the order a body prescribes,
produce the states the body claims they produce. They do.
