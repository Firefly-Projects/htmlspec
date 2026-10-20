# `generate_fixtures` deletes two committed fixtures it does not know about

**A live trap, found by running the generator rather than by reading it, and not this change's to
fix.** Recorded because the next contributor to add a conformance fixture will hit it, and because it
silently removes coverage the project bought deliberately.

## What happens

`cargo run -p htmlspec-conformance --bin generate_fixtures --features generate-fixtures` removes
every directory under `fixtures/conformance/` that is not in its own `FIXTURES` list. Two are not:

```
$ git status --short fixtures/conformance/
 D fixtures/conformance/valid-summary-quoting-the-placeholder/expected.json
 D fixtures/conformance/valid-summary-quoting-the-placeholder/input.html
 D fixtures/conformance/warning-review-placeholder-summary/expected.json
 D fixtures/conformance/warning-review-placeholder-summary/input.html
```

Four files, 146 lines. Both fixtures were added by hand in `0ce5ad5` — "Catch a template's
placeholder summary" (#26) — and neither was added to the generator's list. The deletion is the
generator doing exactly what its code says:

```rust
if !FIXTURES.iter().any(|f| f.name == name) {
    println!("removing stale fixture {name}");
    std::fs::remove_dir_all(&path).expect("the stale fixture is removed");
}
```

That behaviour is deliberate and correct on its own terms — its comment says *"Remove any fixture no
longer declared, so a renamed one does not leave a stale directory the harness would still drive"* —
and the two hand-written fixtures are indistinguishable to it from a renamed one.

## Why this matters more than a file count

**One of the two is the only guard on a decision the project measured and then fixed.**
`valid-summary-quoting-the-placeholder` exists because a substring search would report every document
in this project that discusses the placeholder rule — including the change that introduced it. Its
own change's task 6.3 records that the fixture as first specified *guarded nothing*, because the
harness skips validating a fixture that declares no rules, and that a named test
(`a_summary_quoting_the_placeholder_reports_nothing`) had to be written to make it load-bearing.

So the fixture was found to be inert once, fixed, and is now deletable by a command a contributor is
told to run. The named test is what would fail, loudly — `"{name} is gone, so the decision it pins is
unpinned"` — which is the one thing keeping this from being silent. `warning-review-placeholder-summary`
has no such guard: deleting it makes `every_rule_has_both_a_valid_and_an_invalid_fixture` fail, because
it is the only fixture declaring `review/example-content` at all.

**Neither failure is the one a contributor would understand.** Both read as "your change broke
coverage", when the cause is that a command they ran deleted files they never touched.

## What I did, and what I did not

**Did:** ran the generator to add this change's two fixtures, then restored the two it deleted with
`git checkout`, and re-ran to confirm the restoration survives — it does, because the deletion happens
at the start of each run and nothing else touches them. The committed tree now holds 52 fixtures, of
which the generator's list names 50.

**Did not fix it.** Two fixes are available and both are out of this change's scope:

1. **Add the two to `FIXTURES`.** They are hand-written documents rather than single edits to the
   base, so expressing them as an `edit: fn(String) -> String` means embedding a whole masthead in
   the generator — which is the hand-written near-identical document the generator's header says it
   exists to avoid.
2. **Make the deletion opt-in**, or have it skip a directory carrying a marker. That is a change to
   the fixture harness's contract about what it owns, which is a decision rather than a repair.

The second is the better one and it needs its own argument: the generator currently claims ownership
of the whole directory, and the honest fix is to narrow that claim rather than to widen the list.

## The general form

**A generator that deletes what it does not recognize is a generator that must be the only author of
its output directory.** This one is not, and nothing states the divided ownership. The project already
has the shape of the right answer elsewhere — `ASCENDS_WITHOUT_A_STORE` and `CARRIES_A_STORE` in the
harness are *named* lists of fixtures with unusual properties, so a fixture cannot acquire the property
by accident. A named list of hand-written fixtures the generator must leave alone would be the same
move.
