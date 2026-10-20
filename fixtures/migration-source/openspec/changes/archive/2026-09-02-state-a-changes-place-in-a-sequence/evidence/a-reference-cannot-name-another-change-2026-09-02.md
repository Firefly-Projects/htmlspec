# A reference cannot name another change

Task 1.2, recorded 2026-09-02 against `main` at `595e38c`.

This is the measurement that decides whether the ordering can reuse the reference machinery
already in the projection. It decides against it, and the decision is the command's output
rather than a reading of the rule.

## The rule as written

`crates/htmlspec-validate/src/rules/refs.rs:86`:

```rust
fn addressable_identifiers(projection: &Projection) -> BTreeSet<&str> {
    let mut found: BTreeSet<&str> = projection
        .model
        .units()
        .into_iter()
        .filter_map(|unit| unit.id)
        .collect();
    found.extend(
        projection.model.figures.iter().filter_map(|figure| figure.id.as_deref()),
    );
    found
}
```

The set a reference resolves against is built from `projection.model` — *the same projection*
the reference lives in. There is no parameter through which a second document's identifiers
could arrive. `references_resolve` (`:135`) then reports anything outside that set.

## The measurement

A project with two changes. `contract-first` holds a task whose identifier is
`task-write-the-contract`; `follows-it` holds a task referencing it:

```html
<li class="task" id="task-example" data-change="added" data-state="todo"
    data-refs="task-write-the-contract">
```

The target genuinely exists in the sibling change:

```console
$ grep -c 'task-write-the-contract' htmlspec/changes/contract-first/tasks.html
2
```

Validating the whole project — both documents in one run, one invocation, one project:

```console
$ htmlspec validate
...
validating /private/tmp/seq-probe/htmlspec/changes/follows-it/tasks.html
/private/tmp/seq-probe/htmlspec/changes/follows-it/tasks.html
  error: refs/unresolved at task-example: this task references task-write-the-contract, and
  no unit in the document declares that identifier. A reference that resolves to nothing
  records a relationship to a thing that does not exist

  1 error, 0 warnings: validation failed

6 document(s): 1 error(s), 0 warning(s)
```

**An error, not a warning.** The diagnostic's own wording states the reason it cannot be
relaxed: it reports a relationship "to a thing that does not exist", and from inside one
projection that is exactly what a cross-change reference is. Six documents were read in that
run and the identifier was in one of them, and the rule still reports it — because the rule's
subject is a document, not a run.

## What this rules out, and what it does not

Making a cross-change reference legal would mean teaching `addressable_identifiers` that some
targets live outside the projection. That weakens a rule which is currently exact in both
directions: today, a reference resolves or it does not, and the answer is a function of one
file. Introducing a class of target the rule cannot see would make every unresolved reference
ambiguous between "wrong" and "elsewhere", which is the state the rule exists to prevent.

The design's second reason is independent of this measurement and stands on the accepted
contract rather than on the code: a reference addresses a *unit*, and `artifact-kinds` states
that the four kinds carry units while a change is the thing that holds them. A change is a
directory. Nothing in the projection's vocabulary names one.

So the declaration goes with the change's whole-directory facts. Where that is, in a project
that has no per-change declaration file, is recorded in
`where-the-declaration-goes-2026-09-02.md`.
