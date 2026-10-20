## 1. Re-measure before changing anything

- [x] 1.1 Repeat steps 1 and 2 of `evidence/the-measurement-2026-09-30.md` in a scratch project built with `./target/debug/htmlspec init --tools none`, and record `validate`'s exit status and the `refs/unresolved` diagnostic from each step, checking them against the evidence. Step 1 is a task naming its change specification's `req-example` in `data-refs`. Step 2 is a task naming a requirement that only `htmlspec/specs/measure/spec.html` declares
- [x] 1.2 Re-run the count script recorded in the evidence from `~/projects/html-spec-migration/staged/htmlspec`, record its output, and name any design decision that rests on a number that moved
- [x] 1.3 Run `./target/debug/htmlspec --json validate` in `~/projects/html-spec-migration/staged` under `time`, and record the document count, the error and warning counts by rule, and the wall time as the baseline for task 5.2

## 2. Tests first

- [x] 2.1 Add a CLI test to `crates/htmlspec-cli/tests/cli.rs` for each case below, building each project with `init --tools none` and `new change`, and confirm that every one of them fails on HEAD:
  - a task naming a requirement in its change's specification passes;
  - a task naming a requirement that only an accepted specification declares passes;
  - a decision in the design naming its change's specification passes;
  - a unit in one accepted specification naming another's requirement passes;
  - an archived change's task naming its own specification passes;
  - a task naming a requirement that its change's specification marks `removed`, while the accepted specification holds it unchanged, reports `refs/removed-target` and not `refs/unresolved`;
  - validating by name a tasks document whose task names a requirement in its change's specification, while that specification also carries a requirement with no scenario, succeeds and reports that one document only;
  - a task in a change whose specification needs parser recovery is reported with the specification named as unread;
  - a task in a change with no specification, naming an accepted requirement, passes with nothing reported about the missing specification.
- [x] 2.2 Add a CLI test to the same file for each case below, and confirm that every one of them passes on HEAD:
  - a task naming another change's requirement fails `refs/unresolved`;
  - a fragment link `#req-x` to a unit in the change's specification fails both `refs/unresolved` and `refs/broken-link`;
  - a document outside any project, and a file beneath a change's evidence named on the command line, each naming an identifier that only another document declares, fail `refs/unresolved`;
  - a runbook's tasks document naming an accepted requirement fails `refs/unresolved`.

## 3. The placement and the scope

- [x] 3.1 Add the module of design decision 4 to `crates/htmlspec-validate`, re-exported from `lib.rs`, and give `Subject` the scope and a `placement()` accessor. The module holds `Placement`, `Place`, `Placement::reaches`, `Declared` with its unread list, `Unread`, and `Scope` with `loose`, `gather`, and `placement`. Check with unit tests that `reaches` returns the places in the design's table in order and that `gather` calls its function once for each of them
- [x] 3.2 Pass the scope through `engine::run`. Every caller except `validate_source` passes `Scope::loose()`: `conformance/tests/conformance.rs`, `conformance/tests/runs.rs`, `crates/htmlspec-validate/tests/template_conformance.rs`, the `validate_file` example, and the tests in `engine.rs`, `output.rs`, and `rules/*`. Check with `cargo test --workspace --all-targets` that every test that passed on HEAD still passes, and that the tests from 2.1 still fail
- [x] 3.3 Add `conformance/tests/references.rs`, a project-level suite built through the public API the way `runs.rs` and `topology.rs` build theirs, holding the cases below, and check that they fail before 3.4 and pass after it:
  - one case for each scenario this change adds to document-schema and document-validation, each given as a placement, a document of its own, and what each reached place declares;
  - one property case: every committed conformance fixture reports the same rules at the same locations under every placement with empty places as it does loose.
- [x] 3.4 Make `references_resolve` in `crates/htmlspec-validate/src/rules/refs.rs` do what the list below says, keeping a loose document's message word for word, and check it with unit tests in `refs.rs` (a change's specification shadowing an accepted unit, a withdrawal in the change's specification, an identifier two accepted specifications declare, an unread place, a link reference) and with the suite from 3.3:
  - resolve an attribute-origin reference through the scope's places in order;
  - keep a link-origin reference in its own document;
  - apply `REMOVED_TARGET` in the place that resolves the reference;
  - name the places searched, and any unread document, in the `refs/unresolved` message.
- [x] 3.5 Add `ProjectRoot::placement` to `crates/htmlspec-cli/src/project.rs`, positional and on canonical paths and reusing `is_within_change_evidence` for a change's evidence, and unit-test every row of the design's table and each of these:
  - an archived change's documents;
  - `changes/archive/x.html`;
  - a subdirectory of a change;
  - `htmlspec/x.html`;
  - a file outside `htmlspec/`;
  - a document that is not written yet;
  - a project reached through a symbolic link.
- [x] 3.6 In `crates/htmlspec-cli/src/commands.rs`, make `validate_source` take the document's path and derive the store and the scope from it, reading each other document at most once per run, and move "the accepted specifications" into one function that `merge_state` also calls. Check that the tests from 2.1 pass, that those from 2.2 still pass, and that every caller named in the design's Context goes through it
- [x] 3.7 Restate the one-document rule where it is stated as universal: the `run` help text in `crates/htmlspec-cli/src/main.rs`, the module comment of `crates/htmlspec-cli/src/runs.rs` and the two sentences it writes into every new record, and the assertion message at `crates/htmlspec-cli/tests/run.rs:243`. Leave alone what design decision 13 keeps. Check `htmlspec run --help` and the text of a newly written record, and check that `crates/htmlspec-cli/tests/run.rs` passes

## 4. The instructions

- [x] 4.1 Replace the note at `assets/template-tasks.html:40` with the sentence in design decision 13, update that file's sha256 in `assets/vendor.lock.json`, and check that `cargo test -p htmlspec-assets` and `crates/htmlspec-validate/tests/template_conformance.rs` pass
- [x] 4.2 In `crates/htmlspec-assets/instructions/new.md`, section "The tasks", add one sentence saying where a task's `data-refs` can reach. Regenerate the installed copies with `./target/debug/htmlspec init --tools claude,codex,pi`, remove the stray `htmlspec/` and `spec.nix` it creates, and check that `scripts/check-installed` passes

## 5. Prove it

- [x] 5.1 Repeat task 1.1 and the evidence's plain-link measurement against the built change, and record the results. Steps 1 and 2 must exit 0 with no `refs/*` diagnostic. The plain link must still be accepted with no reference projected
- [x] 5.2 Repeat task 1.3 against the built change, record the wall time against the baseline, and check that the result equals the baseline exactly, because no staged document has a `data-refs` entry that crosses a document
- [x] 5.3 In a scratch copy of the staged tree, add each of the 68 linked identifiers to its unit's `data-refs`, validate, and record the result against what the evidence's counts give:
  - 53 units resolve with no `refs/*` diagnostic: 30 that link to their change's specification, and 23 that link to an accepted specification;
  - task 5.1 of `2026-08-21-distinguish-command-names-from-skill-names` reports `refs/removed-target`;
  - the 2 units that link to another change's specification report `refs/unresolved`.
- [x] 5.4 Mutation-test the change, record which test catches each of these mutations, and add a test for any that nothing catches:
  1. skip the change's specification;
  2. skip the accepted specifications;
  3. try the accepted specifications before the change's specification;
  4. resolve a link-origin reference through the scope;
  5. treat an unread document as an empty set;
  6. apply `REMOVED_TARGET` only in the document itself;
  7. place an archived change's documents as loose;
  8. place a change's evidence as a change document;
  9. let a runbook's document reach the accepted specifications;
  10. let an accepted specification reach only itself.
- [x] 5.5 Run `cargo test --workspace --all-targets`, then `scripts/check`, with every stage green
- [x] 5.6 Record what the implementation revealed in `evidence/`, including the results of 5.1 to 5.4

## 6. Release

- [x] 6.1 Run the release runbook at `runbooks/release/tasks.html`, and report whether it worked as written
- [x] 6.2 Choose the increment, state the reasoning, and flag it for the contributor's confirmation. The design expects `0.22.0`: the next minor after changes 1 to 3, because `validate` now accepts references it rejected and the contract version stays `0.3.0`
- [x] 6.3 Check that `scripts/check-release` passes after archival writes `VERSION` and the hand-written `runbooks.json` beside it
  - `openspec archive resolve-a-reference-across-its-change --yes` (OpenSpec 1.13.1) archived the change as `2026-10-04-resolve-a-reference-across-its-change`, continuing past this one open task. It modified five requirements: one in `openspec/specs/document-schema/spec.md`, two in `document-validation`, one in `runbook-execution`, and one in `toolchain-packaging`. Each merged block equals the delta's word for word, and no other requirement changed
  - `VERSION` holds `0.22.0`. `runbooks.json` is a copy of the previous archive's, because the release runbook is unchanged: the sha256 of `runbooks/release/specification.html` and `tasks.html` still match the inventory's
  - `scripts/check-release`: `ok 28 archived change(s) record a version, and 22 version(s) are in the changelog` and `ok declared version 0.22.0 has a changelog entry`

## 7. Review

- [x] 7.1 Resolve what a review of the pull request found, and record what was taken and what was not
  - **Taken, from the code review of PR #43: a document named by its bare file name was placed loose.** `ProjectRoot::placement` split the path with `Path::parent`, which gives the empty path for a bare file name, while `Reach::scope_for` turned that into the current directory. So `htmlspec validate tasks.html` inside a change found the project and placed the document loose, and reported `refs/unresolved` for a requirement its change's specification declares, while `validate ./tasks.html` passed. Reproduced with the built binary. `project::directory_of` now answers where a document sits for the placement, the scope, and the store walk, which had a third copy of the same logic. `a_document_named_by_its_bare_file_name_is_placed_where_it_sits` covers a change document and an accepted specification named this way, and fails without the fix
  - **Settled with the contributor on 2026-10-04: the command line's private `Declared` enum is now `ContractEffect`.** Two types were named `Declared`, as `what-the-implementation-revealed-2026-10-03.md` records: this change's public `htmlspec_validate::references::Declared`, which is what one place declares, and the enum `see-a-modification-the-merge-did-not-take` added to `crates/htmlspec-cli/src/commands.rs`, which reads a requirement's change state into the three cases the merge comparison judges. `commands.rs` holds both. Everywhere else in it, "declared" means what a document or the project states about itself: a document's kind, the project's version, a unit's change state. The struct uses the word in that sense and the enum does not, because no document declares "establishes" or "carries forward". So the enum moved. The struct keeps its name: it is public, and design decision 4, task 3.1, and `mark-the-words-a-modification-changed`'s design name it, while no design names the enum. **Considered: `Intent`.** Rejected. `new.md`, `continue.md`, and the accepted project-migration specification use "intent" for what a proposal carries, and say a specification must not state it, while the enum reads its value from a change's specification. `ContractEffect` names each case by what the accepted specifications must hold for the requirement to have reached them. The rename changes 21 lines of `commands.rs`, and the variants keep their names, so no behavior changes. The enum's doc comment now says why it is not named for a declaration. The archived tasks of `see-a-modification-the-merge-did-not-take` still name `Declared::of_requirement`, because an archive records what was built
  - **PR #43 was merged before this review question and the archival were settled**, so both land in a follow-up pull request, as `reject-change-markup-inside-a-unit`'s did in PR #40
