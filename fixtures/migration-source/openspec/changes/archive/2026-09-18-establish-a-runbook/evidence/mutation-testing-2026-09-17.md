# Mutation testing: does each test fail when the thing it checks is broken?

A test written for a defect is not the same as a test that fails when the defect is present.
Six mutations, each applied alone and reverted before the next, against
`cargo test -p htmlspec-cli --bin htmlspec` (245 tests clean).

## The results

| # | Mutation | Tests failed |
| --- | --- | --- |
| 1 | `runbooks()` drops its `is_dir` filter, so a loose file counts as a runbook | **0 → then 1** (see below) |
| 2 | `runbooks()` reads `changes()` instead of `runbooks()` | 6 |
| 3 | `Subject::of` ignores the `--runbooks` flag and returns `Changes` | 8 |
| 4 | The document-presence filter always returns true, so every runbook claims both documents | 3 |
| 5 | `ProjectRoot::establish` does not create `runbooks/` | 2 |
| 6 | `ProjectRoot::runbook` joins `changes()` instead of `runbooks()` | 6 |

## Mutation 1 failed nothing, and that is the finding

**Removing the `is_dir` filter failed zero of the nine tests as originally written.** Every
one of them creates only directories under `runbooks/`, so nothing ever presented the
enumeration with a file to reject. The filter was unguarded: a later change could have deleted
it with the suite green, and `runbooks/` would then have listed a stray `README.md` or a
`.DS_Store` as a runbook.

**Fixed** by adding `a_loose_file_beside_the_runbooks_is_not_one`, which writes `NOTES.md`
beside a real runbook and asserts the enumeration returns exactly one row. With the mutation
re-applied that test fails and the other nine still pass — so the guard is isolated to the
property it guards.

This is the same shape the previous change recorded at its task 6.3: a fixture that looked
like a guard and validated nothing, found by mutation rather than by review.

## What mutation 5 shows about the bootstrap decision

Two tests fail and they are the two halves of the decision:
`establishing_a_project_creates_the_directories_it_is_made_of` fails because the place is
absent, and `an_absent_runbooks_directory_enumerates_none` fails because it removes the
directory and expects the *enumeration* to survive that. So creating the place at bootstrap and
tolerating its absence at enumeration are pinned separately, which is right — they are
independent properties and a project bootstrapped by an older binary has the second without
the first.

## What mutation 3 shows about the flag

Eight tests, including `both_flags_resolve_to_the_runbooks`, which is a pure unit test over
`Subject::of` with no filesystem at all. The reason it exists is that a `bool` pair is
representable in a state that means nothing — both set — and `Subject::of` is where that is
decided once. A test asserting all four combinations is cheap and it is the only thing that
would notice the resolution being reordered.

## What was not mutated, and why

**`the_two_document_split_is_forced_by_validation` was not mutated**, because there is nothing
in this change to mutate: it asserts `kind/task-in-specification`, which
`crates/htmlspec-validate/src/rules/` already provides and which this change does not touch.
The test exists to prove the split is forced by an existing rule rather than by this change's
prose — so mutating it would mean breaking a rule from a previous release, which would prove
something about that release rather than about this one.

The same holds for `a_runbooks_documents_validate_as_documents_of_their_kinds`. It runs
`validate_source`, the function the `validate` command runs, over a runbook's two documents. A
mutation that made it pass falsely would have to break validation itself.
