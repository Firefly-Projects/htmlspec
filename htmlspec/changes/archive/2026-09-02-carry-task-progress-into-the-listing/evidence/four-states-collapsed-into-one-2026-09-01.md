# Four states collapsed into one

Task 1.3, recorded 2026-09-01 against `main` at `ee4301c`.

This is the measurement that decides whether a single fraction is a sufficient thing for the
enumeration to carry. It is not.

## What the schema defines

`crates/htmlspec-schema/schema/document.schema.json`, at `/definitions/task`:

```json
"state": {
  "description": "Machine-readable completion state, projected from an attribute rather than from a rendered checkbox.",
  "type": "string",
  "enum": [
    "todo",
    "in-progress",
    "done",
    "dropped"
  ]
}
```

Four states. `state` is among the task's required properties:

```console
$ # /definitions/task
required: ['unit', 'id', 'change', 'title', 'state']
```

`dropped` carries an obligation the others do not — `reason` is "Required when `state` is
`dropped`" — which is the schema recording that abandoning work is a decision someone must
justify, not an absence.

## What the projection permits

`crates/htmlspec-parse/src/model.rs:199` types it more weakly than the schema does:

```rust
pub state: Option<String>,
```

So a fifth case exists in the code that does not exist in the contract: a task whose state the
document did not declare. `open_tasks` handles it deliberately, and its doc comment says so:
"a task declaring no state at all is open — a task nobody has said anything about is not
finished." The counter this change lifts must preserve that reading, and a count per state needs
somewhere to put such a task. Counting it as `todo` would report a state the document never
declared; the honest handling is to keep the distinction the model already makes.

## Where the four become one

`open_tasks` (`crates/htmlspec-cli/src/commands.rs:1414`) filters:

```rust
.filter(|task| !matches!(task.state.as_deref(), Some("done" | "dropped")))
```

That is a two-way partition of a four-way distinction. `todo`, `in-progress`, and
undeclared all land in "open"; `done` and `dropped` both land in "resolved". The filter is
correct for archival, which asks one yes-or-no question — is there outstanding work someone
should be told about before this directory moves — and both losses are invisible to it.

## The two readings a fraction cannot separate

With `total` and `open.len()` as the only numbers, `done` is recovered as
`total - open`, which is what `status` renders. Two pairs of situations then become
indistinguishable:

| Change | `total` | `open` | Renders as | Actually |
| --- | --- | --- | --- | --- |
| A | 24 | 24 | `0/24` | nobody has started it |
| B | 24 | 24 | `0/24` | every task is `in-progress` |
| C | 24 | 4 | `20/24` | four remain to be done |
| D | 24 | 0 | `24/24` | 20 done, 4 `dropped` — finished |

A and B are opposite situations at the same number. C and D differ in the number, but only
because `dropped` was folded into the numerator — D reads as though 24 tasks were completed when
20 were, and the fraction gives no way to see the four that were abandoned. A contributor
choosing what to archive is choosing between exactly these.

So the payload carries the count per state and the human report renders the fraction from it.
The fraction is a rendering, not the record.
