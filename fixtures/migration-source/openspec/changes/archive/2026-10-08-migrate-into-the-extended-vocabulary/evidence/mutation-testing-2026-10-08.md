# Mutation testing

**Date:** 2026-10-08
**Task:** 6.6.

## Every mutation is caught

`mutate.py`, in the appendix, applied each mutation alone, ran the targeted suites with
`--no-fail-fast`, collected what failed, and restored the file from the bytes it saved first. The
four files it touched were hashed before the run and after it, and the hashes match:
`crates/htmlspec-cli/src/migrate.rs`, `crates/htmlspec-cli/src/commands.rs`,
`crates/htmlspec-assets/instructions/migrate.md`, and `~/projects/html-spec-migration/check_draft.py`.

The Rust mutations ran `cargo test -p htmlspec-cli --bin htmlspec --test migrate --test promote`,
after a build. The checker's ran `check_draft_selftest.py` against the working folder and its
staged tree, on the cases whose names hold "strike" or "mark". Mutation 10 of the task, "its marks
check reads one direction only", was run both ways, as 10 and 11, because a check that kept either
direction alone is that mutation.

| # | Mutation | Caught by |
| --- | --- | --- |
| 1 | the survey drops `follows:` | `the_survey_report_names_each_change_that_states_a_predecessor`, `a_predecessor_is_read_as_the_author_wrote_it`, `a_quoted_value_and_a_trailing_comment_are_read_as_the_value`, `the_payload_carries_null_for_a_change_with_no_configuration`, `every_change_configuration_reaches_the_survey` |
| 2 | it omits `configuration` for a change with no file | `the_payload_carries_null_for_a_change_with_no_configuration`, and the nine tests in `tests/migrate.rs` that read the survey, because the harness refuses a payload that omits the field |
| 3 | it reports only the keys it has a name for, so `skip_specs` goes | `a_change_configuration_reports_every_key_it_sets` |
| 4 | it reports `created:`'s value | `a_change_configuration_reports_every_key_it_sets`, `a_creation_date_is_reported_as_a_key_and_its_value_is_not_read`, `a_file_that_states_no_predecessor_states_none`, `the_payload_carries_null_for_a_change_with_no_configuration`, `every_change_configuration_reaches_the_survey` |
| 5 | `migrate.md`'s `## MODIFIED Requirements` row loses `del` | `each_distinction_the_destination_expresses_has_a_row` |
| 6 | the task row loses `[~]` | `each_distinction_the_destination_expresses_has_a_row` |
| 7 | the `.openspec.yaml` row loses `FOLLOWS` | `each_distinction_the_destination_expresses_has_a_row` |
| 8 | promotion's conformance validates each document as loose | `a_staged_reference_across_a_change_is_checked_as_after_promotion`, and `a_dangling_staged_reference_holds_promotion_back`, whose reference to the change's own requirement then fails too, so `refs/unresolved` is reported twice |
| 9 | `check_draft.py`'s strike pattern is restored | self-test: "a strike after the task number is read, so the sync packet's task 4.5 draws no finding" |
| 10 | its marks check reads the current direction only | self-test: "a word marked inserted that the previous text held is reported, in the previous direction only" |
| 11 | its marks check reads the previous direction only | self-test: "a word marked deleted that the delta keeps is reported, in the current direction only" |

**No mutation went uncaught, so no test was added for one.** Mutation 4 was written as the survey
reporting `created: 2026-09-17` among the keys, because the payload has no field a date could reach
otherwise: `ChangeConfiguration` holds `keys` and `follows` only. Mutation 2 removes the field from
each change entry where it is `null`, rather than writing something else in its place. Mutation 8
validates with `Scope::loose()` in place of the scope `Reach` gives the document, which is what a
conformance check that did not place staged documents would do.

The schema half of decision 12's test, `the_destination_expresses_each_distinction_migrate_md_maps`,
was not mutated: it is ignored on this branch. Once its `#[ignore]` is removed in the assembled
stack, renaming any of the four fields it reads in `document.schema.json` fails it. The attribute was
removed on 2026-10-08, when the stack was assembled, and the test runs.

## Appendix: `mutate.py`

Run from the worktree root, outside the Nix shell: `python3 mutate.py`. It is kept in this change's
scratch directory, and the mutations it applies are the eleven rows above, each a single string
replacement in one file, except mutation 2, which is two.

```python
RUST = ["nix", "develop", "-c", "cargo", "test", "-p", "htmlspec-cli", "--bin", "htmlspec",
        "--test", "migrate", "--test", "promote", "--no-fail-fast"]
SELFTEST = ["python3", str(M / "check_draft_selftest.py"), str(M), str(M / "staged"),
            str(S / "source"), str(ROOT / "target/debug/htmlspec"),
            str(M / "tools/project-model/target/release/project-model")]

def failing(output):
    names = re.findall(r"^test (\S+) \.\.\. FAILED$", output, re.M)
    names += re.findall(r"^FAIL (.+)$", output, re.M)
    if re.search(r"^error(\[E\d+\])?:", output, re.M) and not names:
        names.append("<did not compile>")
    return names

for number in chosen:
    title, path, old, new, command = MUTATIONS[number]
    saved = path.read_bytes()
    before = digest(path)
    text = saved.decode("utf-8")
    pairs = old if isinstance(old, list) else [(old, new)]
    for a, b in pairs:
        if text.count(a) != 1:
            raise SystemExit(f"mutation {number}: {a!r} found {text.count(a)} times")
        text = text.replace(a, b, 1)
    path.write_text(text, encoding="utf-8")
    try:
        if command is RUST:
            subprocess.run(["nix", "develop", "-c", "cargo", "build", "-q"], cwd=ROOT,
                           capture_output=True)
        run = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        output = run.stdout + run.stderr
    finally:
        path.write_bytes(saved)
    restored = digest(path) == before
    caught = failing(output)
```

The mutations, as replacements:

```python
1:  "            follows = value_of(rest);\n"  ->  "            let _ = value_of(rest);\n"
2:  ".map(|change| serde_json::json!({ … }))"  ->  ".map(|change| { let mut entry = serde_json::json!({ … });
      if entry[\"configuration\"].is_null() { … o.remove(\"configuration\"); } entry })"
3:  "keys.push(key.to_owned());"  ->  "if [\"schema\", \"created\", \"follows\"].contains(&key) { keys.push(key.to_owned()); }"
4:  "keys.push(key.to_owned());"  ->  "keys.push(if key == \"created\" { format!(\"{key}: {}\", rest.trim()) } else { key.to_owned() });"
5:  "marked with `ins` and `del` against the text it replaced |"  ->  "marked with `ins` against the text it replaced |"
6:  "`- [~]`, or a task struck through, is `dropped` …"  ->  "A task struck through is `dropped` …"
7:  "| `.openspec.yaml` | `follows:` becomes the change's `FOLLOWS`,"  ->  "… becomes the change's predecessor declaration,"
8:  "let report = validate_source(&source, path, &mut reach, Strictness::Default);"
      ->  "let report = { … engine::run(&outcome, &projection, &store_for(path), &Scope::loose(), rules::ALL, Strictness::Default) };"
9:  TASK_LINE's trailing "\.?\s*(~~)?"  ->  "()"
10: "for piece in missing_pieces(previous, …):"  ->  "for piece in missing_pieces(previous, …)[:0]:"
11: "for piece in missing_pieces(current, …):"   ->  "for piece in missing_pieces(current, …)[:0]:"
```

## Found after this run, by review

The code review of PR #52 found what no planned mutation reached: the survey read `follows:` more
simply than YAML does, after a tab before `#`, inside quotes holding ` #`, and after a byte-order mark.
The integration test's independent reader had copied the survey's rule, so it agreed with the
misreading. `a_predecessor_is_read_as_a_yaml_reader_reads_it` reproduced the tab case before the fix,
and the integration test now reads by tokens rather than by the survey's scan.
