# One counter already has two callers

Task 1.2, recorded 2026-09-01 against `main` at `ee4301c`.

## The two functions

Both read the same file and neither is called anywhere else.

`total_tasks` (`crates/htmlspec-cli/src/commands.rs:1293`):

```rust
fn total_tasks(directory: &Path) -> usize {
    let Ok(source) = std::fs::read_to_string(directory.join("tasks.html")) else {
        return 0;
    };
    htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document)
        .model
        .tasks
        .len()
}
```

`open_tasks` (`:1414`):

```rust
fn open_tasks(directory: &Path) -> Vec<String> {
    let Ok(source) = std::fs::read_to_string(directory.join("tasks.html")) else {
        return Vec::new();
    };
    let projection = htmlspec_parse::project(&htmlspec_parse::parse_document(&source).document);
    projection
        .model
        .tasks
        .iter()
        .filter(|task| !matches!(task.state.as_deref(), Some("done" | "dropped")))
        .map(|task| { /* id, else title, else "an unnamed task" */ })
        .collect()
}
```

Every call site, enumerated:

```console
$ grep -n 'total_tasks\|open_tasks' crates/htmlspec-cli/src/commands.rs
1279:    let open = open_tasks(&directory);      # status
1280:    let total = total_tasks(&directory);    # status
1293:fn total_tasks(directory: &Path) -> usize {
1404:        open_tasks: directory,               # findings_for, via Findings
1414:fn open_tasks(directory: &Path) -> Vec<String> {
```

Two callers: `status` at `:1279`–`:1280`, and `findings_for` at `:1404`, which is archival's.
Archival's is the whole of its incomplete-work finding:

```rust
fn findings_for(...) -> Findings {
    Findings {
        open_tasks: open_tasks(directory),
        unmerged: merge_state(project, specification).absent,
    }
}
```

## Two traversals per change today

Worth recording because the change is obliged not to make it three. `status` calls both
functions, and each independently reads `tasks.html`, parses it, and projects it. So one
`status` invocation parses one change's tasks twice. Deriving both numbers from a single
traversal — task 3.2 — removes that as a side effect rather than as its purpose.

## The precedent this extends

`toolchain-packaging`'s accepted requirement `The command surface reports a change's completion`
(`openspec/specs/toolchain-packaging/spec.md:452`) already obliges two of the three to agree:

> Archival SHALL derive its report about incomplete work from this same operation, so that a
> contributor who checks a change before archiving it and the archival that follows cannot
> disagree about what the change holds.

with the scenario at `:462`:

> **WHEN** archival reports incomplete work for a change
> **THEN** the count it reports is the count the completion operation reports for that change

That is satisfied today by construction: one function, two callers. The enumeration becomes a
third caller of the same function, so the change extends an obligation the contract already
states rather than inventing one. What is new is only the arity — the rule was already written.

The same shape is already checked one requirement over, for the merge comparison:
`archival_and_sync_name_the_same_absent_requirements` (`:2601`) asserts `sync` and archival
name the same absent identifiers, with a comment naming the version gate as the reason. Task
3.4 is that test's counterpart for the counts.
