# What the listing discards

Task 1.1, recorded 2026-09-01 against `main` at `ee4301c`.

## A note on the line numbers

The tasks document cites `active_changes` at `commands.rs:1019`, `status` at `:1088`,
`total_tasks` at `:1148`, and `open_tasks` at `:1269`. Three of the four have drifted, because
PR #16 inserted `sync` between `list` and `status`. The cited positions were correct when the
tasks were written and are recorded here as they stand now, since evidence that points at the
wrong line is worse than evidence that points at a name:

| Cited | Actual at `ee4301c` | Item |
| --- | --- | --- |
| `:1019` | `:1019` | `fn active_changes` |
| — | `:1039` | `pub fn list` |
| `:1088` | `:1233` | `pub fn status` |
| `:1148` | `:1293` | `fn total_tasks` |
| `:1269` | `:1414` | `fn open_tasks` |

The names are stable and the offsets are not, which is itself the argument for citing a function
rather than a line.

## What the enumeration produces

`active_changes` (`crates/htmlspec-cli/src/commands.rs:1019`) returns `Vec<String>`. Its return
type is the measurement: a change is a name and nothing else by the time `list` sees it.

```rust
fn active_changes(project: &ProjectRoot) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(project.changes()) else {
        return Vec::new();
    };
    let mut found: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "archive")
        .collect();
    found.sort();
    found
}
```

One `read_dir`, no file is opened. `list` (`:1039`) then emits those strings under a label:

```rust
let payload = serde_json::json!({ label: items });
```

So the structured payload for this repository's four active changes is:

```json
{
  "changes": [
    "carry-task-progress-into-the-listing",
    "distinguish-a-requirement-from-an-example",
    "migrate-an-openspec-project"
  ]
}
```

The human report is the same strings, one per line. `list --specs` takes the other arm and
enumerates document paths under `specs/`, which this change leaves alone.

## What `status` already computes for one change

`status` (`:1233`) resolves one change and computes two things the enumeration does not:

```rust
let open = open_tasks(&directory);
let total = total_tasks(&directory);
```

and emits:

```rust
"tasks": { "total": total, "open": open.len(), "openTasks": open },
```

alongside per-artifact presence. Its human report renders one line:

```rust
"  {} of {total} task(s) done or dropped\n",
total.saturating_sub(open.len())
```

## The gap, stated as a difference

For one change, `status` knows `{ total, open, openTasks }` and which artifacts exist. For every
change, `list` knows the name. The information the enumeration throws away is not information it
would have to invent — it is information the same file computes twenty lines below, for one
subject at a time.

An agent that wants to select among four changes on any basis other than alphabetical order must
therefore make five calls: one `list`, then one `status` per name it got back. That is the cost
this change removes, and it is the cost the accepted requirement did not intend, since it says
the enumeration exists so a workflow can select what it acts on.
