## 1. The check, before the fix

- [x] 1.1 Add a test in `crates/htmlspec-cli/src/install.rs` that installs for every tool in `TOOLS` into a scratch directory, reads every skill and command file written, extracts the `name:` value, and asserts it matches lowercase letters, digits, and hyphens only. Read the produced files rather than the table: the malformed value is assembled by a format string at write time, so a check on `workflow.name` and `workflow.directory` — both already conforming — cannot see it
- [x] 1.2 Make the failure name the file, the offending value, and the character that failed. Across 35 generated files, "a name is malformed" is not actionable
- [x] 1.3 Run the test and **watch it fail** on the 21 command files three tools currently receive, each carrying `/htmlspec:…`, `$htmlspec-…`, or `/htmlspec-…`. A check that has never failed proves nothing, and this is the only naturally occurring failing case
- [x] 1.4 Record the failure output as evidence, including the count of failing files per tool. That count is what shows the check reads every file rather than the first one

## 2. Separate the name from the invocation

- [x] 2.1 Change `write_commands` in `install.rs` to use `workflow.directory` for the `name:` field instead of `tool.invocation(workflow.name)`, so a command file and its skill file declare one identity. `workflow.name` is rejected for this: a flat command directory already needs the `htmlspec-` prefix on the *file* name to avoid colliding with another tool's unrelated `apply.md`, and a bare `apply` in the name field reintroduces that ambiguity
- [x] 2.2 Leave the rest of the command frontmatter as it is — description, allowed-tools, and `generatedBy` are unaffected
- [x] 2.3 Record in a comment why the invocation is not the name, and where the invocation does belong. The next reader will see a name field that no longer matches what they type and will be tempted to put it back
- [x] 2.4 Confirm `Tool::invocation` now has exactly one non-test caller, `report_next_command`. That single-producer property is the decision, not a side effect of it
- [x] 2.5 Re-run the test from task 1 and confirm it passes

## 3. Nothing a contributor types has changed

- [x] 3.1 Confirm the command **file names** are untouched: `.claude/commands/htmlspec/apply.md` and `.cursor/commands/htmlspec-apply.md` are what determine the invocation, and this change must not move them
- [x] 3.2 Confirm `report_next_command` still prints `/htmlspec:new`, `$htmlspec-new`, `/htmlspec-new`, and the AGENTS.md sentence, grouped per distinct spelling. The closing message is the one place that teaches the spelling and it must be unchanged
- [x] 3.3 Run the full suite and confirm no existing test regressed. `every_invocation_names_its_workflow` in `tools.rs` in particular must still pass — it tests `Tool::invocation` itself, which this change does not alter

## 4. Prove a renamed field does not break loading

- [x] 4.1 Regenerate this repository's own instructions with `htmlspec init --tools claude,codex,pi`, as the previous change did, and confirm the 21 command files now declare `htmlspec-*-change` names — **14** here, not 21: this repository selects `claude,codex,pi` and Codex is skills-only. 21 is the all-tools figure, confirmed separately by the new test
- [x] 4.2 Confirm the seven `/htmlspec:*` commands still load and are still invocable by their old spelling. This is the risk that would make the change wrong rather than merely different, and it must be checked by loading rather than by reasoning: if a loader derived the command name from the `name:` field, the spelling would change
- [x] 4.3 Record the result as evidence, including the vendored `opsx` comparison — `name: "OPSX: Apply"` registers as `/opsx:apply`, which is neither the name nor derivable from it, so the file path is what determines the command. That comparison is the reason to expect this to hold, and the load is the proof
- [x] 4.4 Confirm the AGENTS.md region is untouched by the regeneration, since it carries no frontmatter and this change does not reach it

## 5. The contract

- [x] 5.1 Apply the modified requirement from the delta to `openspec/specs/project-bootstrap/spec.md` as a partial update, leaving every requirement and scenario the change does not name intact
- [x] 5.2 Confirm the requirement keeps its nine existing paragraphs and eight scenarios, and gains the two paragraphs and four scenarios about generated names — the applied requirement is textually identical to the delta's, and the diff to `openspec/specs/` is 20 insertions and 0 deletions. The requirement had **eight** paragraphs, not nine, so it now carries ten and twelve scenarios
- [x] 5.3 Confirm no other accepted requirement is touched. This change modifies one requirement in one capability
