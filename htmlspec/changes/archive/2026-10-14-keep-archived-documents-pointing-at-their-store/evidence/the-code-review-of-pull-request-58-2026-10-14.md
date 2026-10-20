# The code review of pull request #58, and what was done

**Date:** 2026-10-14
**Tree:** this change's branch at `5bcf9c7`, the head the review read, and then the repairs below.

The review (medium) found two low-severity issues. Each is repaired, and each repair has a test that
failed against `5bcf9c7` for the reason the review gave.

## 1. The path check compared two paths as text

**Found.** `Store::resolve` compared the reference's directory with the store's, each relative to the
document with only `.` and `..` dropped. A reference that climbs out and back in, such as
`../../../htmlspec/assets/` from `htmlspec/changes/<name>/`, names the store's directory, and a
browser loads the store's bytes through it, but it was reported as leading elsewhere. `format` keeps
it as written, so validation failed and formatting could not repair it.

**Decided.** A reference leads to the store when the directory it names, followed from the document's
directory, is the store's directory. Both directories are made absolute, and every `.` and `..` is
resolved lexically, as a browser resolves a relative URL. The filesystem is never asked, so a link or
a case-insensitive filesystem does not change the verdict: `../../ASSETS/` is rejected everywhere,
because it loads the store on macOS and nothing on a Linux machine running the checks. The design's
decision 4 and its risk "A path through a symbolic link" record this. `Store::seen_through(path)`
became `Store::seen_from(directory)`.

**Before the repair** (`cargo test -p htmlspec-cli --test cli`):

```
test a_reference_spelled_in_another_case_does_not_reach_the_store ... ok
test a_reference_that_climbs_out_and_back_in_reaches_the_store ... FAILED
  ../../../htmlspec/assets/ reaches the store and was rejected:
  error: asset/unresolved at /link[href]: <link href="../../../htmlspec/assets/sha256-03627137bcbb81c9/htmlspec.css">
  leads to ../../../htmlspec/assets/ from this document, and the project's store is at ../../assets/. …
```

The case test passed before and after, because it pins the decision, which both checks give.

**After the repair**, both pass, as do three new unit tests in `htmlspec-assets`:
`seen_from_a_document_a_path_out_and_back_in_reaches_the_store` (four spellings, including one that
climbs past the project and back in by its own directory's name),
`seen_from_a_document_a_path_in_another_case_leads_elsewhere`, and
`seen_from_a_document_an_absolute_path_leads_where_it_names`, which also accepts a path from the root
that names the store's directory.

The verdicts were compared again with 0.29.1's over the same trees, and they are what
`what-a-path-check-would-change-2026-10-14.md` records: no change under `htmlspec/`, the fixtures, the
runbook, or `README.html`, and the same six documents under `openspec/`.

## 2. A staged copy written part-way was refused as an edit

**Found.** `write_new` opens a staged copy and then writes it, which is not atomic. A run killed
part-way before the move leaves a copy whose digest is not the journal's. The next run's recovery
refused it with "Somebody has edited it since", rather than rolling back, and `perform` then refused
on the leftover copy, so the contributor had to delete it by hand, on a false premise.

**Decided.** Before the move, recovery removes a staged copy whatever it holds, and does not ask
about its digest. Nothing but archival writes a `.<name>.archiving` file, and before the move it is
derived from a document still in place and untouched, so removing it loses nothing. After the move
the copy is about to be published, every copy was written whole before the move could happen, and a
digest that disagrees is an edit, which is still refused. Writing through a temporary file and a
rename was not taken: it moves the partial write to the temporary file, which recovery and the
leftover check would then have to know by a second name. The design's decision 3 records this.

The test `an_edited_staged_document_is_refused`, which edited a staged copy in a change that had not
moved and expected a refusal, asserted the behaviour this repairs. It became
`an_edited_staged_document_is_refused_after_the_move`, where the refusal still applies and still
leaves the edit and the archived document untouched.

**Before the repair** (`cargo test -p htmlspec-cli --test archive -- staged`):

```
test an_interruption_while_a_document_was_staged_rolls_back ... FAILED
  recovery refused a staged copy archival was writing when it stopped:
  htmlspec: an interrupted archival left a journal at …/htmlspec/.archive-journal.json naming
  …/htmlspec/changes/torn-staged/.proposal.html.archiving, and that file does not hold what the
  transaction intended. Somebody has edited it since, …
test an_edited_staged_document_is_refused_after_the_move ... ok
```

**After the repair**, both pass. The first leaves half of the rewrite as the staged copy, as a run
killed mid-write does, and asserts that recovery rolls back, removes it, leaves the document as it
was, and that the next `htmlspec archive --yes` archives the change with the full rewrite in place.

## Not changed

The runbook inventory and the recorded version are written into the change the same way, by an
ordinary write, so a run killed while writing one leaves the same kind of partial file, and recovery
refuses it as an edit. That is 0.29.1's behaviour, and `a_conflicting_edit_to_an_owned_file_is_refused`
pins the refusal for an edited `VERSION` in a change that has not moved. The review did not raise it,
and this change leaves it as it was.

## The checks after both repairs

`nix develop -c scripts/check` passed `format`, `lint`, `test` (1461 tests, none failing), `doctest`,
`self-conformance`, `self-format`, `self-runbooks`, `self-runbooks-format`, and `self-assets`. Its four
Python stages failed inside the shell with `tool 'python3' not found`, as they do on this machine, and
passed outside it with the shell's `cargo` on the path: `prompt` (4 cases), `installed` (45 files),
`release-fixtures` (23 cases), and `release` (36 archived changes, and 0.30.0 has a changelog entry).
